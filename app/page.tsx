'use client'

import { useState, useEffect } from 'react'
import { DndContext, DragEndEvent, DragOverlay, DragStartEvent } from '@dnd-kit/core'
import { KanbanBoard } from '@/components/KanbanBoard'
import { ChatInterface } from '@/components/ChatInterface'
import { TicketModal } from '@/components/TicketModal'
import { TicketDetailModal } from '@/components/TicketDetailModal'
import { useSocket } from '@/hooks/useSocket'
import {
  Ticket,
  TicketStatus,
  StructuredLogMessage,
  CodeAnalysisCompleteMessage,
  CodeAnalysisErrorMessage,
} from '@/types/ticket'

export default function Home() {
  const [tickets, setTickets] = useState<Ticket[]>([])
  const [selectedTicket, setSelectedTicket] = useState<Ticket | null>(null)
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [draggedTicket, setDraggedTicket] = useState<Ticket | null>(null)

  // Detail modal state
  const [detailModalOpen, setDetailModalOpen] = useState(false)
  const [selectedTicketForDetail, setSelectedTicketForDetail] = useState<Ticket | null>(null)

  const { socket, isConnected } = useSocket()

  useEffect(() => {
    // Load initial tickets with proper structure
    const initialTickets: Ticket[] = [
      {
        id: '1',
        title: 'Hiểu flow đăng nhập user',
        description: 'Cần hiểu cách hệ thống xử lý đăng nhập user',
        status: 'todo',
        createdAt: new Date(),
        codeContext: 'auth/login.js',
        isAnalyzing: false,
        logs: [],
      },
      {
        id: '2',
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
  }, [])

  useEffect(() => {
    if (socket) {
      const handleMessage = (event: MessageEvent) => {
        try {
          const data = JSON.parse(event.data)
          console.log('📩 Nhận được message từ backend:', data)

          switch (data.message_type) {
            case 'structured-log':
              handleStructuredLog(data as StructuredLogMessage)
              break

            case 'code-analysis-complete':
              handleAnalysisComplete(data as CodeAnalysisCompleteMessage)
              break

            case 'code-analysis-error':
              handleAnalysisError(data as CodeAnalysisErrorMessage)
              break

            default:
              console.log('Unknown message type:', data.message_type)
          }
        } catch (error) {
          console.error('Lỗi parse message:', error)
        }
      }

      socket.addEventListener('message', handleMessage)

      return () => {
        socket.removeEventListener('message', handleMessage)
      }
    }
  }, [socket])

  const handleStructuredLog = (message: StructuredLogMessage) => {
    const { log } = message

    console.log('📝 Structured log:', log)

    setTickets((prev) =>
      prev.map((ticket) => {
        if (ticket.id === log.ticketId || ticket.id === log.ticket_id) {
          return {
            ...ticket,
            logs: [...ticket.logs, log],
          }
        }
        return ticket
      })
    )

    // Update selected ticket for detail modal if it's open
    setSelectedTicketForDetail((prev) => {
      if (prev && (prev.id === log.ticketId || prev.id === log.ticket_id)) {
        return {
          ...prev,
          logs: [...prev.logs, log],
        }
      }
      return prev
    })
  }

  const handleAnalysisComplete = (message: CodeAnalysisCompleteMessage) => {
    console.log('✅ Analysis complete:', message)

    setTickets((prev) =>
      prev.map((ticket) => {
        if (ticket.id === message.ticket_id) {
          return {
            ...ticket,
            analysisResult: message.content,
            isAnalyzing: false,
          }
        }
        return ticket
      })
    )

    // Update selected ticket for detail modal
    setSelectedTicketForDetail((prev) => {
      if (prev && prev.id === message.ticket_id) {
        return {
          ...prev,
          analysisResult: message.content,
          isAnalyzing: false,
        }
      }
      return prev
    })
  }

  const handleAnalysisError = (message: CodeAnalysisErrorMessage) => {
    console.error('❌ Analysis error:', message)

    setTickets((prev) =>
      prev.map((ticket) => {
        if (ticket.id === message.ticket_id) {
          return {
            ...ticket,
            isAnalyzing: false,
          }
        }
        return ticket
      })
    )

    // Update selected ticket for detail modal
    setSelectedTicketForDetail((prev) => {
      if (prev && prev.id === message.ticket_id) {
        return {
          ...prev,
          isAnalyzing: false,
        }
      }
      return prev
    })
  }

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

    setTickets((prev) =>
      prev.map((ticket) =>
        ticket.id === ticketId ? { ...ticket, status: newStatus } : ticket
      )
    )

    // Trigger Cursor Agent khi ticket được chuyển sang in-progress
    if (newStatus === 'in-progress') {
      const ticket = tickets.find((t) => t.id === ticketId)
      if (ticket) {
        handleStartAnalysis(ticketId)
      }
    }
  }

  const handleCreateTicket = (ticket: Omit<Ticket, 'id' | 'createdAt' | 'isAnalyzing' | 'logs'>) => {
    const newTicket: Ticket = {
      ...ticket,
      id: Date.now().toString(),
      createdAt: new Date(),
      isAnalyzing: false,
      logs: [],
    }
    setTickets((prev) => [...prev, newTicket])
    setIsModalOpen(false)
  }

  const handleEditTicket = (ticket: Ticket) => {
    setSelectedTicket(ticket)
    setIsModalOpen(true)
  }

  const handleCardClick = (ticket: Ticket) => {
    setSelectedTicketForDetail(ticket)
    setDetailModalOpen(true)
  }

  const handleStartAnalysis = (ticketId: string) => {
    const ticket = tickets.find((t) => t.id === ticketId)

    if (!ticket) {
      console.error('Ticket not found:', ticketId)
      return
    }

    if (socket && socket.readyState === WebSocket.OPEN) {
      console.log('🚀 Starting analysis for ticket:', ticketId)

      // Mark ticket as analyzing and clear previous logs
      setTickets((prev) =>
        prev.map((t) =>
          t.id === ticketId
            ? { ...t, isAnalyzing: true, logs: [], analysisResult: undefined }
            : t
        )
      )

      // Update detail modal if open
      setSelectedTicketForDetail((prev) => {
        if (prev && prev.id === ticketId) {
          return {
            ...prev,
            isAnalyzing: true,
            logs: [],
            analysisResult: undefined,
          }
        }
        return prev
      })

      // Send WebSocket message to backend
      const message = {
        type: 'start-code-analysis',
        ticketId,
        codeContext: ticket.codeContext || '',
        question: ticket.description,
      }

      socket.send(JSON.stringify(message))
      console.log('📤 Sent analysis request:', message)
    } else {
      console.error('WebSocket not connected')
    }
  }

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <h1 className="text-2xl font-bold text-gray-900">QA Chatbot MVP</h1>
            <div className="flex items-center space-x-4">
              <div
                className={`px-3 py-1 rounded-full text-sm ${
                  isConnected ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                }`}
              >
                {isConnected ? '✅ Đã kết nối' : '❌ Mất kết nối'}
              </div>
              <button
                onClick={() => setIsModalOpen(true)}
                className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors"
              >
                Tạo Ticket Mới
              </button>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          <div className="lg:col-span-2">
            <DndContext onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
              <KanbanBoard
                tickets={tickets}
                onEditTicket={handleEditTicket}
                onCardClick={handleCardClick}
              />
              <DragOverlay>
                {draggedTicket ? (
                  <div className="kanban-card dragging">
                    <h3 className="font-semibold">{draggedTicket.title}</h3>
                    <p className="text-sm text-gray-600">{draggedTicket.description}</p>
                  </div>
                ) : null}
              </DragOverlay>
            </DndContext>
          </div>

          <div className="lg:col-span-1">
            <ChatInterface selectedTicket={selectedTicket} isConnected={isConnected} />
          </div>
        </div>
      </main>

      <TicketModal
        isOpen={isModalOpen}
        onClose={() => {
          setIsModalOpen(false)
          setSelectedTicket(null)
        }}
        onSubmit={handleCreateTicket}
        ticket={selectedTicket}
      />

      <TicketDetailModal
        ticket={selectedTicketForDetail}
        isOpen={detailModalOpen}
        onClose={() => {
          setDetailModalOpen(false)
          setSelectedTicketForDetail(null)
        }}
        onStartAnalysis={handleStartAnalysis}
      />
    </div>
  )
}
