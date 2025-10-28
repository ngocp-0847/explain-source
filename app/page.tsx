'use client'

import { useEffect } from 'react'
import { DndContext, DragEndEvent, DragStartEvent } from '@dnd-kit/core'
import { KanbanBoard } from '@/components/KanbanBoard'
import { ChatInterface } from '@/components/ChatInterface'
import { TicketFormDialog } from '@/components/TicketFormDialog'
import { TicketDetailDialog } from '@/components/TicketDetailDialog'
import { useTicketStore } from '@/stores/ticketStore'
import { useWebSocketStore } from '@/stores/websocketStore'
import { useUIStore } from '@/stores/uiStore'
import { Ticket, TicketStatus, StructuredLogMessage, CodeAnalysisCompleteMessage, CodeAnalysisErrorMessage, isValidLogMessageType } from '@/types/ticket'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'

export default function Home() {
  const tickets = useTicketStore(state => state.tickets)
  const setTickets = useTicketStore(state => state.setTickets)
  const addTicket = useTicketStore(state => state.addTicket)
  const updateTicketStatus = useTicketStore(state => state.updateTicketStatus)
  const addTicketLog = useTicketStore(state => state.addTicketLog)
  const setAnalysisResult = useTicketStore(state => state.setAnalysisResult)
  const setTicketAnalyzing = useTicketStore(state => state.setTicketAnalyzing)
  const startAnalysis = useTicketStore(state => state.startAnalysis)

  const openTicketModal = useUIStore(state => state.openTicketModal)
  const closeTicketModal = useUIStore(state => state.closeTicketModal)
  const isTicketModalOpen = useUIStore(state => state.isTicketModalOpen)
  const selectedTicketForEdit = useUIStore(state => state.selectedTicketForEdit)

  const openDetailModal = useUIStore(state => state.openDetailModal)
  const closeDetailModal = useUIStore(state => state.closeDetailModal)
  const isDetailModalOpen = useUIStore(state => state.isDetailModalOpen)
  const selectedTicketForDetail = useUIStore(state => state.selectedTicketForDetail)
  const setDraggedTicket = useUIStore(state => state.setDraggedTicket)
  const draggedTicket = useUIStore(state => state.draggedTicket)

  const { isConnected, subscribe, connect, send } = useWebSocketStore()

  // Load tickets from backend on mount
  useEffect(() => {
    if (isConnected) {
      send({ type: 'load-tickets' })
    }
  }, [isConnected, send])

  // Fallback: Set initial tickets if no tickets from backend after 1 second
  useEffect(() => {
    const timer = setTimeout(() => {
      if (tickets.length === 0 && isConnected) {
        // Set initial tickets and sync to backend
        const initialTickets: Ticket[] = [
          {
            id: crypto.randomUUID(),
            title: 'Hiểu flow đăng nhập user',
            description: 'Cần hiểu cách hệ thống xử lý đăng nhập user',
            status: 'todo',
            createdAt: new Date(),
            codeContext: 'auth/login.js',
            isAnalyzing: false,
            logs: [],
          },
          {
            id: crypto.randomUUID(),
            title: 'Phân tích API payment',
            description: 'Tìm hiểu flow thanh toán trong hệ thống',
            status: 'todo',
            createdAt: new Date(),
            codeContext: 'api/payment.js',
            isAnalyzing: false,
            logs: [],
          },
        ]
        setTickets(initialTickets)
        
        // Sync each ticket to backend
        initialTickets.forEach(ticket => {
          send({
            type: 'create-ticket',
            id: ticket.id,
            title: ticket.title,
            description: ticket.description,
            status: ticket.status,
            codeContext: ticket.codeContext || undefined,
          })
        })
      }
    }, 1000)
    
    return () => clearTimeout(timer)
  }, [tickets.length, setTickets, send, isConnected])

  useEffect(() => {
    // Connect WebSocket
    connect('ws://localhost:9000/ws')

    // Subscribe to WebSocket messages
    const unsubscribe = subscribe((data) => {
      switch (data.message_type) {
        case 'structured-log':
          const logMessage = data as StructuredLogMessage
          const log = logMessage.log
          
          // Validate và fix messageType nếu không hợp lệ
          if (!isValidLogMessageType(log.messageType)) {
            console.warn(`Invalid messageType received: ${log.messageType}, falling back to 'system'`)
            log.messageType = 'system'
          }
          
          addTicketLog(log.ticketId || log.ticket_id, log)
          break

        case 'code-analysis-complete':
          const completeMsg = data as CodeAnalysisCompleteMessage
          setAnalysisResult(completeMsg.ticket_id, completeMsg.content)
          break

        case 'code-analysis-error':
          const errorMsg = data as CodeAnalysisErrorMessage
          setTicketAnalyzing(errorMsg.ticket_id, false)
          break

        case 'tickets-loaded':
          try {
            const loadedTickets = JSON.parse(data.content)
            if (Array.isArray(loadedTickets) && loadedTickets.length > 0) {
              setTickets(loadedTickets)
              console.log('✅ Loaded', loadedTickets.length, 'tickets from backend')
            }
          } catch (e) {
            console.error('Failed to parse tickets:', e)
          }
          break

        case 'ticket-created':
          try {
            const newTicket = JSON.parse(data.content)
            // Only add if not already in local state
            if (!tickets.find(t => t.id === newTicket.id)) {
              addTicket(newTicket)
            }
          } catch (e) {
            console.error('Failed to parse ticket:', e)
          }
          break

        case 'ticket-status-updated':
          const ticketId = data.ticket_id
          const newStatus = data.content as TicketStatus
          updateTicketStatus(ticketId, newStatus)
          break
      }
    })

    return unsubscribe
  }, [connect, subscribe, addTicketLog, setAnalysisResult, setTicketAnalyzing, setTickets, tickets, addTicket, updateTicketStatus])

  const handleDragStart = (event: DragStartEvent) => {
    const ticket = tickets.find((t) => t.id === event.active.id)
    setDraggedTicket(ticket || null)
  }

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    setDraggedTicket(null)

    if (!over) return

    const ticketId = active.id as string
    const newStatus = over.id as TicketStatus

    // 1. Update local state
    updateTicketStatus(ticketId, newStatus)

    // 2. Sync to backend FIRST
    send({
      type: 'update-ticket-status',
      ticketId,
      status: newStatus
    })

    // 3. Start analysis with small delay to ensure backend processes status update
    if (newStatus === 'in-progress') {
      setTimeout(() => {
        startAnalysis(ticketId, send)
      }, 150)
    }
  }

  const handleEditTicket = (ticket: Ticket) => {
    useUIStore.getState().setSelectedTicketForEdit(ticket)
    openTicketModal()
  }

  const handleCardClick = (ticket: Ticket) => {
    openDetailModal(ticket.id)
  }

  const handleStartAnalysis = (ticketId: string) => {
    startAnalysis(ticketId, send)
  }

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <h1 className="text-2xl font-bold text-gray-900">QA Chatbot MVP</h1>
            <div className="flex items-center space-x-4">
              <Badge
                variant={isConnected ? 'default' : 'destructive'}
                className={
                  isConnected
                    ? 'bg-green-100 text-green-800 hover:bg-green-100'
                    : 'bg-red-100 text-red-800 hover:bg-red-100'
                }
              >
                {isConnected ? '✅ Đã kết nối' : '❌ Mất kết nối'}
              </Badge>
              <Button onClick={openTicketModal}>
                Tạo Ticket Mới
              </Button>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <div className="lg:col-span-2">
            <DndContext onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
              <KanbanBoard
                onEditTicket={handleEditTicket}
                onCardClick={handleCardClick}
              />
              {draggedTicket ? (
                <div className="kanban-card dragging fixed pointer-events-none z-50">
                  <h3 className="font-semibold">{draggedTicket.title}</h3>
                  <p className="text-sm text-gray-600">{draggedTicket.description}</p>
                </div>
              ) : null}
            </DndContext>
          </div>

          <div className="lg:col-span-1">
            <ChatInterface isConnected={isConnected} />
          </div>
        </div>
      </main>

      <TicketFormDialog />

      <TicketDetailDialog onStartAnalysis={handleStartAnalysis} />
    </div>
  )
}