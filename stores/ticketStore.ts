import { create } from 'zustand'
import { Ticket, TicketStatus, StructuredLog } from '@/types/ticket'
import { ticketApi } from '@/lib/api'

interface TicketStore {
  tickets: Ticket[]
  analysisTimeouts: Map<string, NodeJS.Timeout>
  setTickets: (tickets: Ticket[]) => void
  addTicket: (ticket: Ticket) => void
  updateTicket: (id: string, updates: Partial<Ticket>) => void
  updateTicketStatus: (id: string, status: TicketStatus) => void
  setTicketAnalyzing: (id: string, isAnalyzing: boolean) => void
  addTicketLog: (ticketId: string, log: StructuredLog) => void
  setAnalysisResult: (ticketId: string, result: string) => void
  startAnalysis: (ticketId: string, sendMessage: (message: any) => void) => void
  loadTicketLogs: (ticketId: string) => Promise<void>
  stopAnalysisTimeout: (ticketId: string) => void
}

export const useTicketStore = create<TicketStore>((set, get) => ({
  tickets: [],
  analysisTimeouts: new Map(),

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

  updateTicketStatus: async (id, status) => {
    // Update local state immediately
    set((state) => ({
      tickets: state.tickets.map((ticket) =>
        ticket.id === id ? { ...ticket, status } : ticket
      ),
    }))
    
    // Update on server
    try {
      await ticketApi.updateStatus(id, status)
    } catch (error) {
      console.error('Failed to update ticket status on server:', error)
      // Revert local state on error
      const ticket = get().tickets.find(t => t.id === id)
      if (ticket) {
        // Keep the optimistic update for now
      }
    }
  },

  setTicketAnalyzing: (id, isAnalyzing) => {
    // Clear timeout if stopping analysis
    if (!isAnalyzing) {
      const timeouts = get().analysisTimeouts
      if (timeouts.has(id)) {
        clearTimeout(timeouts.get(id)!)
        timeouts.delete(id)
      }
    }
    
    set((state) => ({
      tickets: state.tickets.map((ticket) =>
        ticket.id === id ? { ...ticket, isAnalyzing } : ticket
      ),
    }))
  },

  addTicketLog: (ticketId, log) =>
    set((state) => ({
      tickets: state.tickets.map((ticket) =>
        ticket.id === ticketId || ticket.id === log.ticketId
          ? { ...ticket, logs: [...ticket.logs, log] }
          : ticket
      ),
    })),

  setAnalysisResult: (ticketId, result) => {
    // Clear timeout if exists
    const timeouts = get().analysisTimeouts
    if (timeouts.has(ticketId)) {
      clearTimeout(timeouts.get(ticketId)!)
      timeouts.delete(ticketId)
    }
    
    set((state) => ({
      tickets: state.tickets.map((ticket) =>
        ticket.id === ticketId
          ? { ...ticket, analysisResult: result, isAnalyzing: false }
          : ticket
      ),
    }))
  },

  startAnalysis: (ticketId: string, sendMessage: (message: any) => void) => {
    const ticket = get().tickets.find((t) => t.id === ticketId)
    if (!ticket) return

    // Clear any existing timeout for this ticket
    const timeouts = get().analysisTimeouts
    if (timeouts.has(ticketId)) {
      clearTimeout(timeouts.get(ticketId)!)
      timeouts.delete(ticketId)
    }

    // Mark ticket as analyzing and clear previous logs
    set((state) => ({
      tickets: state.tickets.map((t) =>
        t.id === ticketId
          ? { ...t, isAnalyzing: true, logs: [], analysisResult: undefined }
          : t
      ),
    }))

    // Set timeout fallback (5 minutes)
    const timeoutId = setTimeout(() => {
      console.warn(`⚠️ Analysis timeout for ticket ${ticketId} after 5 minutes`)
      
      // Stop analyzing and add timeout log
      set((state) => ({
        tickets: state.tickets.map((t) =>
          t.id === ticketId
            ? { 
                ...t, 
                isAnalyzing: false,
                logs: [...t.logs, {
                  id: `timeout-${Date.now()}`,
                  ticketId,
                  messageType: 'system' as const,
                  content: '⏰ Phân tích tự động dừng sau 5 phút (timeout)',
                  timestamp: new Date().toISOString(),
                }]
              }
            : t
        ),
      }))
      
      // Remove timeout from map
      const currentTimeouts = get().analysisTimeouts
      currentTimeouts.delete(ticketId)
    }, 5 * 60 * 1000) // 5 minutes

    // Store timeout ID
    timeouts.set(ticketId, timeoutId)

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

  loadTicketLogs: async (ticketId: string) => {
    try {
      const logs = await ticketApi.getLogs(ticketId)
      set((state) => ({
        tickets: state.tickets.map((ticket) =>
          ticket.id === ticketId 
            ? { 
                ...ticket, 
                logs: logs.map((log: any) => ({
                  id: log.id,
                  ticketId: log.ticket_id,
                  messageType: log.message_type,
                  content: log.content,
                  rawLog: log.raw_log,
                  metadata: log.metadata ? JSON.parse(log.metadata) : undefined,
                  timestamp: log.timestamp,
                }))
              }
            : ticket
        ),
      }))
    } catch (error) {
      console.error('Failed to load logs:', error)
    }
  },

  stopAnalysisTimeout: (ticketId: string) => {
    const timeouts = get().analysisTimeouts
    if (timeouts.has(ticketId)) {
      clearTimeout(timeouts.get(ticketId)!)
      timeouts.delete(ticketId)
      console.log(`🛑 Cleared timeout for ticket ${ticketId}`)
    }
  },
}))

