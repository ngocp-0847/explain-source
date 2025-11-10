'use client'

import { useEffect, useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import { 
  DndContext, 
  DragEndEvent, 
  DragStartEvent,
  PointerSensor,
  useSensor,
  useSensors,
  closestCorners,
  DragOverlay
} from '@dnd-kit/core'
import { KanbanBoard } from '@/components/KanbanBoard'
import { ChatPopup } from '@/components/ChatPopup'
import { TicketFormDialog } from '@/components/TicketFormDialog'
import { TicketDetailDialog } from '@/components/TicketDetailDialog'
import { useTicketStore } from '@/stores/ticketStore'
import { useProjectStore } from '@/stores/projectStore'
import { useWebSocketStore } from '@/stores/websocketStore'
import { useUIStore } from '@/stores/uiStore'
import { Ticket, TicketStatus, StructuredLogMessage, CodeAnalysisCompleteMessage, CodeAnalysisErrorMessage, isValidLogMessageType, RawStructuredLog } from '@/types/ticket'
import { projectApi, ticketApi } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { ArrowLeft, Settings2, GripVertical } from 'lucide-react'
import { ProjectFormDialog } from '@/components/ProjectFormDialog'

export default function ProjectDetailPage() {
  const params = useParams()
  const router = useRouter()
  const projectId = params.projectId as string

  // Loading state for projects
  const [isLoading, setIsLoading] = useState(true)
  const [projectNotFound, setProjectNotFound] = useState(false)

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
  const selectedTicketIdForDetail = useUIStore(state => state.selectedTicketIdForDetail)
  const setDraggedTicket = useUIStore(state => state.setDraggedTicket)
  const draggedTicket = useUIStore(state => state.draggedTicket)
  const isProjectSettingsOpen = useUIStore(state => state.isProjectSettingsOpen)
  const openProjectSettings = useUIStore(state => state.openProjectSettings)
  const closeProjectSettings = useUIStore(state => state.closeProjectSettings)

  const { isConnected, subscribe, connect, send } = useWebSocketStore()

  // Configure drag sensors
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8, // 8px movement required to start drag
      },
    })
  )

  // Load project detail from API
  useEffect(() => {
    const loadProject = async () => {
      try {
        const data = await projectApi.get(projectId)
        // Map API response to frontend format
        const mappedProject = {
          ...data,
          directoryPath: data.directory_path,
          createdAt: data.created_at,
          updatedAt: data.updated_at,
        }
        useProjectStore.getState().addProject(mappedProject)
        useProjectStore.getState().selectProject(projectId)
        setIsLoading(false)
      } catch (error) {
        console.error('Failed to load project:', error)
        setProjectNotFound(true)
        setIsLoading(false)
      }
    }
    loadProject()
  }, [projectId])

  // Load tickets from API
  useEffect(() => {
    const loadTickets = async () => {
      try {
        const data = await ticketApi.list(projectId)
        setTickets(data.map((t: any) => ({
          id: t.id,
          projectId: t.project_id, // Map snake_case to camelCase
          title: t.title,
          description: t.description,
          status: t.status,
          createdAt: new Date(t.created_at),
          updatedAt: t.updated_at ? new Date(t.updated_at) : undefined,
          codeContext: t.code_context,
          analysisResult: t.analysis_result,
          isAnalyzing: t.is_analyzing,
          logs: [], // Khởi tạo empty array
        })))
      } catch (error) {
        console.error('Failed to load tickets:', error)
      }
    }
    if (!isLoading) {
      loadTickets()
    }
  }, [projectId, isLoading, setTickets])

  // Connect WebSocket (only for real-time logs)
  useEffect(() => {
    connect('ws://localhost:9000/ws')

    // Subscribe to WebSocket messages (only for analysis logs)
    const unsubscribe = subscribe((data) => {
      switch (data.message_type) {
        case 'structured-log':
          const logMessage = data as StructuredLogMessage
          const rawLog: RawStructuredLog = logMessage.log
          
          // Transform snake_case fields to camelCase
          const transformedLog = {
            id: rawLog.id,
            ticketId: rawLog.ticket_id,
            messageType: rawLog.message_type,
            content: rawLog.content,
            rawLog: rawLog.raw_log,
            metadata: rawLog.metadata,
            timestamp: rawLog.timestamp,
          }
          
          if (!isValidLogMessageType(transformedLog.messageType)) {
            console.warn(`Invalid messageType received: ${transformedLog.messageType}, falling back to 'system'`)
            transformedLog.messageType = 'system'
          }
          
          addTicketLog(transformedLog.ticketId, transformedLog)
          
          // Stop analyzing when result log is received
          if (transformedLog.messageType === 'result') {
            setTicketAnalyzing(transformedLog.ticketId, false)
          }
          break

        case 'code-analysis-complete':
          const completeMsg = data as CodeAnalysisCompleteMessage
          setAnalysisResult(completeMsg.ticket_id, completeMsg.content)
          break

        case 'code-analysis-error':
          const errorMsg = data as CodeAnalysisErrorMessage
          setTicketAnalyzing(errorMsg.ticket_id, false)
          break
      }
    })

    return unsubscribe
  }, [connect, subscribe, addTicketLog, setAnalysisResult, setTicketAnalyzing])

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

  const handleStopAnalysis = async (ticketId: string) => {
    await useTicketStore.getState().stopAnalysis(ticketId)
  }

  const handleOpenSettings = () => {
    if (selectedProject) {
      openProjectSettings(selectedProject)
    }
  }

  // Add timeout fallback
  useEffect(() => {
    const timeout = setTimeout(() => {
      if (isLoading) {
        setIsLoading(false)
        setProjectNotFound(true)
      }
    }, 5000)
    
    return () => clearTimeout(timeout)
  }, [isLoading])

  // Show loading state
  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-gray-900 mx-auto mb-4"></div>
          <h1 className="text-xl font-semibold text-gray-900">Đang tải project...</h1>
        </div>
      </div>
    )
  }

  // Show error if project not found
  if (projectNotFound || !selectedProject) {
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
    <div className="min-h-screen bg-gray-100" suppressHydrationWarning>
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
              <Button variant="outline" onClick={handleOpenSettings}>
                <Settings2 className="w-4 h-4 mr-2" />
                Settings
              </Button>
              <Button onClick={openTicketModal}>
                Tạo Ticket Mới
              </Button>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <DndContext 
          sensors={sensors}
          collisionDetection={closestCorners}
          onDragStart={handleDragStart} 
          onDragEnd={handleDragEnd}
        >
          <KanbanBoard
            onEditTicket={handleEditTicket}
            onCardClick={handleCardClick}
          />
          <DragOverlay>
            {draggedTicket ? (
              <Card className="kanban-card opacity-90 shadow-2xl rotate-3 scale-105">
                <CardContent className="p-2">
                  <div className="flex items-start">
                    <GripVertical className="w-5 h-5 text-gray-400 mr-1.5" />
                    <div>
                      <h4 className="font-medium text-gray-900 mb-1">{draggedTicket.title}</h4>
                      <p className="text-sm text-gray-600">{draggedTicket.description}</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ) : null}
          </DragOverlay>
        </DndContext>
      </main>

      <TicketFormDialog />
      <TicketDetailDialog onStartAnalysis={handleStartAnalysis} onStopAnalysis={handleStopAnalysis} />
      <ProjectFormDialog 
        isOpen={isProjectSettingsOpen}
        onClose={closeProjectSettings}
        project={selectedProject}
      />
      <ChatPopup isConnected={isConnected} />
    </div>
  )
}

