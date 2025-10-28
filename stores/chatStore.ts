import { create } from 'zustand'

export interface ChatMessage {
  id: string
  type: 'user' | 'bot'
  content: string
  timestamp: Date
}

interface ChatStore {
  messages: ChatMessage[]
  addMessage: (message: ChatMessage) => void
  clearMessages: () => void
  setTyping: (isTyping: boolean) => void
  isTyping: boolean
}

export const useChatStore = create<ChatStore>((set) => ({
  messages: [],
  isTyping: false,

  addMessage: (message) =>
    set((state) => ({
      messages: [...state.messages, message],
    })),

  clearMessages: () => set({ messages: [] }),

  setTyping: (isTyping) => set({ isTyping }),
}))

