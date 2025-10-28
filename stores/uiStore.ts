import { create } from 'zustand'
import { Ticket, Project } from '@/types/ticket'

interface UIStore {
  isTicketModalOpen: boolean
  selectedTicketForEdit: Ticket | null
  isDetailModalOpen: boolean
  selectedTicketIdForDetail: string | null
  draggedTicket: Ticket | null
  isProjectSettingsOpen: boolean
  selectedProjectForEdit: Project | null
  
  openTicketModal: () => void
  closeTicketModal: () => void
  setSelectedTicketForEdit: (ticket: Ticket | null) => void
  
  openDetailModal: (ticketId: string | null) => void
  closeDetailModal: () => void
  
  setDraggedTicket: (ticket: Ticket | null) => void
  
  openProjectSettings: (project: Project) => void
  closeProjectSettings: () => void
}

export const useUIStore = create<UIStore>((set) => ({
  isTicketModalOpen: false,
  selectedTicketForEdit: null,
  isDetailModalOpen: false,
  selectedTicketIdForDetail: null,
  draggedTicket: null,
  isProjectSettingsOpen: false,
  selectedProjectForEdit: null,

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

  openProjectSettings: (project) =>
    set({
      isProjectSettingsOpen: true,
      selectedProjectForEdit: project,
    }),
  closeProjectSettings: () =>
    set({
      isProjectSettingsOpen: false,
      selectedProjectForEdit: null,
    }),
}))

