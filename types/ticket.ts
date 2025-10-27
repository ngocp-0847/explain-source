export type TicketStatus = 'todo' | 'in-progress' | 'done'

export interface Ticket {
  id: string
  title: string
  description: string
  status: TicketStatus
  createdAt: Date
  codeContext?: string
  analysisResult?: string
}

export interface CodeAnalysis {
  ticketId: string
  codeContext: string
  question: string
  result: string
  logs: string[]
  timestamp: Date
}