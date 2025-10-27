'use client'

import { useDroppable } from '@dnd-kit/core'
import { SortableContext, verticalListSortingStrategy } from '@dnd-kit/sortable'
import { Ticket, TicketStatus } from '@/types/ticket'
import { KanbanCard } from './KanbanCard'

interface KanbanColumnProps {
  status: TicketStatus
  title: string
  color: string
  tickets: Ticket[]
  onEditTicket: (ticket: Ticket) => void
  onCardClick: (ticket: Ticket) => void
}

export function KanbanColumn({
  status,
  title,
  color,
  tickets,
  onEditTicket,
  onCardClick
}: KanbanColumnProps) {
  const { setNodeRef } = useDroppable({
    id: status,
  })

  return (
    <div className={`kanban-column ${color}`}>
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-gray-700">{title}</h3>
        <span className="bg-white text-gray-600 px-2 py-1 rounded-full text-sm">
          {tickets.length}
        </span>
      </div>
      
      <SortableContext 
        items={tickets.map(ticket => ticket.id)} 
        strategy={verticalListSortingStrategy}
      >
        <div ref={setNodeRef} className="space-y-3">
          {tickets.map(ticket => (
            <KanbanCard
              key={ticket.id}
              ticket={ticket}
              onEdit={onEditTicket}
              onClick={onCardClick}
            />
          ))}
        </div>
      </SortableContext>
    </div>
  )
}