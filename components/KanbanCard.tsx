'use client'

import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { Ticket } from '@/types/ticket'
import { EditIcon, CodeIcon } from 'lucide-react'

interface KanbanCardProps {
  ticket: Ticket
  onEdit: (ticket: Ticket) => void
}

export function KanbanCard({ ticket, onEdit }: KanbanCardProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: ticket.id })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`kanban-card ${isDragging ? 'dragging' : ''} ${
        ticket.status === 'in-progress' ? 'in-progress' : ''
      }`}
      {...attributes}
      {...listeners}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <h4 className="font-medium text-gray-900 mb-1">{ticket.title}</h4>
          <p className="text-sm text-gray-600 mb-2">{ticket.description}</p>
          {ticket.codeContext && (
            <div className="flex items-center text-xs text-blue-600 mb-2">
              <CodeIcon className="w-3 h-3 mr-1" />
              {ticket.codeContext}
            </div>
          )}
          <div className="text-xs text-gray-500">
            {ticket.createdAt.toLocaleDateString('vi-VN')}
          </div>
        </div>
        <button
          onClick={(e) => {
            e.stopPropagation()
            onEdit(ticket)
          }}
          className="ml-2 p-1 hover:bg-gray-100 rounded transition-colors"
        >
          <EditIcon className="w-4 h-4 text-gray-500" />
        </button>
      </div>
      
      {ticket.analysisResult && (
        <div className="mt-3 p-2 bg-green-50 border border-green-200 rounded text-sm">
          <div className="font-medium text-green-800 mb-1">Kết quả phân tích:</div>
          <div className="text-green-700">{ticket.analysisResult}</div>
        </div>
      )}
    </div>
  )
}