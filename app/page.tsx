'use client'

import { useState, useEffect } from 'react'
import { DndContext, DragEndEvent, DragOverlay, DragStartEvent } from '@dnd-kit/core'
import { SortableContext, verticalListSortingStrategy } from '@dnd-kit/sortable'
import { KanbanBoard } from '@/components/KanbanBoard'
import { ChatInterface } from '@/components/ChatInterface'
import { TicketModal } from '@/components/TicketModal'
import { useSocket } from '@/hooks/useSocket'
import { Ticket, TicketStatus } from '@/types/ticket'

export default function Home() {
  const [tickets, setTickets] = useState<Ticket[]>([])
  const [selectedTicket, setSelectedTicket] = useState<Ticket | null>(null)
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [draggedTicket, setDraggedTicket] = useState<Ticket | null>(null)
  
  const { socket, isConnected } = useSocket()

  useEffect(() => {
    // Load initial tickets
    const initialTickets: Ticket[] = [
      {
        id: '1',
        title: 'Hiểu flow đăng nhập user',
        description: 'Cần hiểu cách hệ thống xử lý đăng nhập user',
        status: 'todo',
        createdAt: new Date(),
        codeContext: 'auth/login.js'
      },
      {
        id: '2',
        title: 'Phân tích API payment',
        description: 'Tìm hiểu flow thanh toán trong hệ thống',
        status: 'todo',
        createdAt: new Date(),
        codeContext: 'api/payment.js'
      }
    ]
    setTickets(initialTickets)
  }, [])

  useEffect(() => {
    if (socket) {
      const handleMessage = (event: MessageEvent) => {
        try {
          const data = JSON.parse(event.data)
          console.log('Nhận được message từ backend:', data)
          
          if (data.message_type === 'code-analysis-complete') {
            console.log('Nhận được phân tích code:', data)
            // Cập nhật UI với kết quả phân tích
          } else if (data.message_type === 'cursor-agent-log') {
            console.log('Cursor Agent log:', data.content)
            // Hiển thị log real-time
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

  const handleDragStart = (event: DragStartEvent) => {
    const ticket = tickets.find(t => t.id === event.active.id)
    setDraggedTicket(ticket || null)
  }

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    setDraggedTicket(null)

    if (!over) return

    const ticketId = active.id as string
    const newStatus = over.id as TicketStatus

    setTickets(prev => 
      prev.map(ticket => 
        ticket.id === ticketId 
          ? { ...ticket, status: newStatus }
          : ticket
      )
    )

    // Trigger Cursor Agent khi ticket được chuyển sang in-progress
    if (newStatus === 'in-progress') {
      const ticket = tickets.find(t => t.id === ticketId)
      if (ticket && socket && socket.readyState === WebSocket.OPEN) {
        const message = {
          type: 'start-code-analysis',
          ticketId,
          codeContext: ticket.codeContext,
          question: ticket.description
        }
        socket.send(JSON.stringify(message))
      }
    }
  }

  const handleCreateTicket = (ticket: Omit<Ticket, 'id' | 'createdAt'>) => {
    const newTicket: Ticket = {
      ...ticket,
      id: Date.now().toString(),
      createdAt: new Date()
    }
    setTickets(prev => [...prev, newTicket])
    setIsModalOpen(false)
  }

  const handleEditTicket = (ticket: Ticket) => {
    setSelectedTicket(ticket)
    setIsModalOpen(true)
  }

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <h1 className="text-2xl font-bold text-gray-900">
              QA Chatbot MVP
            </h1>
            <div className="flex items-center space-x-4">
              <div className={`px-3 py-1 rounded-full text-sm ${
                isConnected ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
              }`}>
                {isConnected ? 'Đã kết nối' : 'Mất kết nối'}
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
            <ChatInterface 
              selectedTicket={selectedTicket}
              isConnected={isConnected}
            />
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
    </div>
  )
}