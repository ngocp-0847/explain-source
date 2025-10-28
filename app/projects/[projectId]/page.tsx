'use client'

import { useEffect } from 'react'
import { useParams, useRouter } from 'next/navigation'
import { DndContext, DragEndEvent, DragStartEvent } from '@dnd-kit/core'
import { KanbanBoard } from '@/components/KanbanBoard'
import { ChatInterface } from '@/components/ChatInterface'
import { TicketFormDialog } from '@/components/TicketFormDialog'
import { TicketDetailDialog } from '@/components/TicketDetailDialog'
import { useTicketStore } from '@/stores/ticketStore'
import { useProjectStore } from '@/stores/projectStore'
import { useWebSocketStore } from '@/stores/websocketStore'
import { useUIStore } from '@/stores/uiStore'
import { Ticket, TicketStatus, StructuredLogMessage, CodeAnalysisCompleteMessage, CodeAnalysisErrorMessage, isValidLogMessageType } from '@/types/ticket'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ArrowLeft, Settings2 } from 'lucide-react'

export default function ProjectDetailPage() {
  const params = useParams()
  const router = useRouter()
  const projectId = params.projectId as string

  const tickets = useTicketStore(state => state.tickets)
  const setTickets = useTicketStore(state => state.setTickets)
  const addTicket = useTicketStore(state => state.addTicket)
  const updateTicketStatus = useTicketStore(state => state.updateTicketStatus)
  const addTicketLog = useTicketStore(state => state.addTicketLog)
  const setAnalysisResult = useTicketStore(state => state.setAnalysisResult)
  const setTicketAnalyzing = useTicketStore(state => state.setTicketAnalyzing)
  const startAnalysis = useTicketStore(state => state.startAnalysis)

  const { projects, selectProject } = useProjectStore()
  const selectedProject = projects.find(p => p.id === projectId)

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

  // Select project on mount
  useEffect(() => {
    selectProject(projectId)
  }, [projectId, selectProject])

  // Load tickets from backend on mount
  useEffect(() => {
    if (isConnected) {
      send({ type: 'load-tickets', projectId })
    }
  }, [isConnected, projectId, send])

  // Connect WebSocket
  useEffect(() => {
    connect('ws://localhost:9000/ws')

    // Subscribe to WebSocket messages
    const unsubscribe = subscribe((data) => {
      switch (data.message_type) {
        case 'structured-log':
          const logMessage = data as StructuredLogMessage
          const log = logMessage.log
          
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
            if (Array.isArray(loadedTickets)) {
              // Filter tickets for this project
              const projectTickets = loadedTickets.filter((t: any) => t.project_id === projectId)
              setTickets(projectTickets.map((t: any) => ({
                ...t,
                createdAt: new Date(t.created_at),
                updatedAt: t.updated_at ? new Date(t.updated_at) : undefined,
              })))
              console.log('✅ Loaded', projectTickets.length, 'tickets for project')
            }
          } catch (e) {
            console.error('Failed to parse tickets:', e)
          }
          break

        case 'project-created':
          try {
            const newProject = JSON.parse(data.content)
            useProjectStore.getState().addProject(newProject)
          } catch (e) {
            console.error('Failed to parse project:', e)
          }
          break

        case 'projects-loaded':
          try {
            const loadedProjects = JSON.parse(data.content)
            if (Array.isArray(loadedProjects)) {
              useProjectStore.getState().setProjects(loadedProjects)
            }
          } catch (e) {
            console.error('Failed to parse projects:', e)
          }
          break
      }
    })

    return unsubscribe
  }, [connect, subscribe, addTicketLog, setAnalysisResult, setTicketAnalyzing, setTickets, send, projectId])

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

    updateTicketStatus(ticketId, newStatus)

    send({
      type: 'update-ticket-status',
      ticketId,
      status: newStatus
    })

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

  if (!selectedProject) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-gray-900 mb-4">Project không tồn tại</h1>
          <Button onClick={() => router.push('/projects')}>
            Quay lại Projects
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center gap-4">
              <Button variant="ghost" onClick={() => router.push('/projects')} size="sm">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back
              </Button>
              <div>
                <h1 className="text-2xl font-bold text-gray-900">{selectedProject.name}</h1>
                <p className="text-sm text-gray-600">{selectedProject.description || 'Không có mô tả'}</p>
              </div>
            </div>
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
        <div className="mb-4 p-4 bg-white rounded-lg shadow-sm">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-semibold text-gray-900">Directory Path</h3>
              <p className="text-sm text-gray-600 font-mono">{selectedProject.directoryPath}</p>
            </div>
            <Button variant="outline" size="sm">
              <Settings2 className="w-4 h-4 mr-2" />
              Settings
            </Button>
          </div>
        </div>

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

