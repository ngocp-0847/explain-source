'use client'

import { TicketStatus } from '@/types/ticket'
import { KanbanColumn } from './KanbanColumn'
import { useTicketStore } from '@/stores/ticketStore'

interface KanbanBoardProps {
  onEditTicket: (ticket: any) => void
  onCardClick: (ticket: any) => void
}

const columns: { id: TicketStatus; title: string; color: string }[] = [
  { id: 'todo', title: 'Cần Làm', color: 'bg-yellow-50' },
  { id: 'in-progress', title: 'Đang Xử Lý', color: 'bg-blue-50' },
  { id: 'done', title: 'Hoàn Thành', color: 'bg-green-50' }
]

export function KanbanBoard({ onEditTicket, onCardClick }: KanbanBoardProps) {
  const tickets = useTicketStore(state => state.tickets)

  return (
    <div className="flex space-x-6 overflow-x-auto pb-4">
      {columns.map(column => (
        <KanbanColumn
          key={column.id}
          status={column.id}
          title={column.title}
          color={column.color}
          tickets={tickets.filter(ticket => ticket.status === column.id)}
          onEditTicket={onEditTicket}
          onCardClick={onCardClick}
        />
      ))}
    </div>
  )
}