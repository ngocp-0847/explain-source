'use client'

import { Ticket, TicketStatus } from '@/types/ticket'
import { KanbanColumn } from './KanbanColumn'
import { PlusIcon } from 'lucide-react'

interface KanbanBoardProps {
  tickets: Ticket[]
  onEditTicket: (ticket: Ticket) => void
}

const columns: { id: TicketStatus; title: string; color: string }[] = [
  { id: 'todo', title: 'Cần Làm', color: 'bg-gray-100' },
  { id: 'in-progress', title: 'Đang Xử Lý', color: 'bg-blue-100' },
  { id: 'done', title: 'Hoàn Thành', color: 'bg-green-100' }
]

export function KanbanBoard({ tickets, onEditTicket }: KanbanBoardProps) {
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
        />
      ))}
    </div>
  )
}