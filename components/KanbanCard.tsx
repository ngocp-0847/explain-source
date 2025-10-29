'use client'

import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { Ticket } from '@/types/ticket'
import { EditIcon, CodeIcon, GripVertical, Loader2 } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'

interface KanbanCardProps {
  ticket: Ticket
  onEdit: (ticket: Ticket) => void
  onClick: (ticket: Ticket) => void
}

export function KanbanCard({ ticket, onEdit, onClick }: KanbanCardProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ 
    id: ticket.id,
    data: {
      type: 'ticket',
      ticket,
    }
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  const handleCardClick = (e: React.MouseEvent) => {
    const target = e.target as HTMLElement
    if (!target.closest('button') && !target.closest('.drag-handle')) {
      onClick(ticket)
    }
  }

  return (
    <Card
      ref={setNodeRef}
      style={style}
      className={`kanban-card ${isDragging ? 'dragging' : ''} ${
        ticket.status === 'in-progress' ? 'in-progress' : ''
      } mb-3 cursor-move hover:shadow-md transition-shadow`}
    >
      <CardContent className="p-2">
        <div className="flex items-start">
          {/* Drag Handle */}
          <div {...listeners} {...attributes} className="drag-handle cursor-grab active:cursor-grabbing mr-1.5">
            <GripVertical className="w-5 h-5 text-gray-400" />
          </div>

          {/* Click Area */}
          <div className="flex-1 cursor-pointer" onClick={handleCardClick}>
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <h4 className="font-medium text-gray-900 mb-0.5">{ticket.title}</h4>
                <p className="text-sm text-gray-600 mb-1">{ticket.description}</p>
                {ticket.codeContext && (
                  <div className="flex items-center text-xs text-blue-600 mb-1">
                    <CodeIcon className="w-3 h-3 mr-1" />
                    {ticket.codeContext}
                  </div>
                )}
                <div className="text-xs text-gray-500">
                  {ticket.createdAt.toLocaleDateString('vi-VN')}
                </div>
              </div>
              <Button
                variant="ghost"
                size="icon"
                className="ml-2"
                onClick={(e) => {
                  e.stopPropagation()
                  onEdit(ticket)
                }}
              >
                <EditIcon className="w-4 h-4 text-gray-500" />
              </Button>
            </div>

            {ticket.isAnalyzing && (
              <div className="mt-2 flex items-center text-blue-600 text-sm">
                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                Đang phân tích...
              </div>
            )}

            {ticket.analysisResult && !ticket.isAnalyzing && (
              <Badge success className="mt-2">
                ✅ Đã phân tích
              </Badge>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  )
}