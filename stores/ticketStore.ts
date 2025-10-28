import { create } from 'zustand'
import { Ticket, TicketStatus, StructuredLog } from '@/types/ticket'

interface TicketStore {
  tickets: Ticket[]
  setTickets: (tickets: Ticket[]) => void
  addTicket: (ticket: Ticket) => void
  updateTicket: (id: string, updates: Partial<Ticket>) => void
  updateTicketStatus: (id: string, status: TicketStatus) => void
  setTicketAnalyzing: (id: string, isAnalyzing: boolean) => void
  addTicketLog: (ticketId: string, log: StructuredLog) => void
  setAnalysisResult: (ticketId: string, result: string) => void
  startAnalysis: (ticketId: string, sendMessage: (message: any) => void) => void
}

export const useTicketStore = create<TicketStore>((set, get) => ({
  tickets: [],

  setTickets: (tickets) => set({ tickets }),

  addTicket: (ticket) =>
    set((state) => ({
      tickets: [...state.tickets, ticket],
    })),

  updateTicket: (id, updates) =>
    set((state) => ({
      tickets: state.tickets.map((ticket) =>
        ticket.id === id ? { ...ticket, ...updates } : ticket
      ),
    })),

  updateTicketStatus: (id, status) =>
    set((state) => ({
      tickets: state.tickets.map((ticket) =>
        ticket.id === id ? { ...ticket, status } : ticket
      ),
    })),

  setTicketAnalyzing: (id, isAnalyzing) =>
    set((state) => ({
      tickets: state.tickets.map((ticket) =>
        ticket.id === id ? { ...ticket, isAnalyzing } : ticket
      ),
    })),

  addTicketLog: (ticketId, log) =>
    set((state) => ({
      tickets: state.tickets.map((ticket) =>
        ticket.id === ticketId || ticket.id === log.ticketId
          ? { ...ticket, logs: [...ticket.logs, log] }
          : ticket
      ),
    })),

  setAnalysisResult: (ticketId, result) =>
    set((state) => ({
      tickets: state.tickets.map((ticket) =>
        ticket.id === ticketId
          ? { ...ticket, analysisResult: result, isAnalyzing: false }
          : ticket
      ),
    })),

  startAnalysis: (ticketId: string, sendMessage: (message: any) => void) => {
    const ticket = get().tickets.find((t) => t.id === ticketId)
    if (!ticket) return

    // Mark ticket as analyzing and clear previous logs
    set((state) => ({
      tickets: state.tickets.map((t) =>
        t.id === ticketId
          ? { ...t, isAnalyzing: true, logs: [], analysisResult: undefined }
          : t
      ),
    }))

    // Send analysis request via WebSocket
    const message = {
      type: 'start-code-analysis',
      ticketId,
      codeContext: ticket.codeContext || '',
      question: ticket.description,
      projectId: ticket.projectId,
    }

    sendMessage(message)
    console.log('🚀 Starting analysis for ticket:', ticketId, 'in project:', ticket.projectId)
  },
}))

