'use client'

import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { Ticket } from '@/types/ticket'
import { EditIcon, CodeIcon } from 'lucide-react'

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
  } = useSortable({ id: ticket.id })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`kanban-card cursor-pointer ${isDragging ? 'dragging' : ''} ${
        ticket.status === 'in-progress' ? 'in-progress' : ''
      }`}
      onClick={(e) => {
        // Only trigger onClick if not clicking on edit button
        const target = e.target as HTMLElement
        if (!target.closest('button')) {
          onClick(ticket)
        }
      }}
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
      
      {ticket.isAnalyzing && (
        <div className="mt-3 flex items-center text-blue-600 text-sm">
          <svg className="animate-spin h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Đang phân tích...
        </div>
      )}

      {ticket.analysisResult && !ticket.isAnalyzing && (
        <div className="mt-3 p-2 bg-green-50 border border-green-200 rounded text-sm">
          <div className="font-medium text-green-800 mb-1">✅ Đã phân tích</div>
          <div className="text-green-700 text-xs">Click để xem kết quả chi tiết</div>
        </div>
      )}
    </div>
  )
}