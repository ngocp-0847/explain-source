import { create } from 'zustand'
import { Ticket } from '@/types/ticket'

interface UIStore {
  isTicketModalOpen: boolean
  selectedTicketForEdit: Ticket | null
  isDetailModalOpen: boolean
  selectedTicketIdForDetail: string | null
  draggedTicket: Ticket | null
  
  openTicketModal: () => void
  closeTicketModal: () => void
  setSelectedTicketForEdit: (ticket: Ticket | null) => void
  
  openDetailModal: (ticketId: string | null) => void
  closeDetailModal: () => void
  
  setDraggedTicket: (ticket: Ticket | null) => void
}

export const useUIStore = create<UIStore>((set) => ({
  isTicketModalOpen: false,
  selectedTicketForEdit: null,
  isDetailModalOpen: false,
  selectedTicketIdForDetail: null,
  draggedTicket: null,

  openTicketModal: () => set({ isTicketModalOpen: true }),
  closeTicketModal: () => set({ isTicketModalOpen: false, selectedTicketForEdit: null }),
  
  setSelectedTicketForEdit: (ticket) =>
    set({ selectedTicketForEdit: ticket }),

  openDetailModal: (ticketId) =>
    set({ 
      isDetailModalOpen: ticketId !== null, 
      selectedTicketIdForDetail: ticketId 
    }),
  closeDetailModal: () =>
    set({ 
      isDetailModalOpen: false, 
      selectedTicketIdForDetail: null 
    }),

  setDraggedTicket: (ticket) => set({ draggedTicket: ticket }),
}))

