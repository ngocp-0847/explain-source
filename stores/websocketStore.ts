import { create } from 'zustand'

type MessageHandler = (data: any) => void

interface WebSocketStore {
  socket: WebSocket | null
  isConnected: boolean
  messageHandlers: Set<MessageHandler>
  connect: (url: string) => void
  disconnect: () => void
  send: (message: any) => void
  subscribe: (handler: MessageHandler) => () => void
}

export const useWebSocketStore = create<WebSocketStore>((set, get) => {
  let reconnectTimeoutRef: NodeJS.Timeout | null = null

  const connect = (url: string) => {
    if (get().socket?.readyState === WebSocket.OPEN) {
      return
    }

    const connectWebSocket = () => {
      try {
        const newSocket = new WebSocket(url)

        newSocket.onopen = () => {
          set({ isConnected: true, socket: newSocket })
          console.log('✅ Đã kết nối với Rust backend')
          
          if (reconnectTimeoutRef) {
            clearTimeout(reconnectTimeoutRef)
            reconnectTimeoutRef = null
          }
        }

        newSocket.onclose = () => {
          set({ isConnected: false })
          console.log('❌ Mất kết nối với Rust backend')

          // Auto-reconnect after 3 seconds
          reconnectTimeoutRef = setTimeout(() => {
            console.log('🔄 Đang thử kết nối lại...')
            connectWebSocket()
          }, 3000)
        }

        newSocket.onerror = (error) => {
          console.error('❌ WebSocket error:', error)
          set({ isConnected: false })
        }

        newSocket.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data)
            console.log('📩 Nhận được message:', data)

            // Call all subscribed handlers
            get().messageHandlers.forEach((handler) => {
              try {
                handler(data)
              } catch (error) {
                console.error('Error in message handler:', error)
              }
            })
          } catch (error) {
            console.error('❌ Error parsing message:', error)
          }
        }

        set({ socket: newSocket })
      } catch (error) {
        console.error('❌ Lỗi tạo WebSocket:', error)
        set({ isConnected: false })
      }
    }

    connectWebSocket()
  }

  return {
    socket: null as WebSocket | null,
    isConnected: false,
    messageHandlers: new Set<MessageHandler>(),
    connect,
    disconnect: () => {
      if (reconnectTimeoutRef) {
        clearTimeout(reconnectTimeoutRef)
      }
      const socket = get().socket
      if (socket) {
        socket.close()
        set({ socket: null, isConnected: false })
      }
    },
    send: (message: any) => {
      const socket = get().socket
      if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify(message))
      } else {
        console.error('❌ WebSocket not connected')
      }
    },
    subscribe: (handler: MessageHandler) => {
      set((state) => ({
        messageHandlers: new Set(state.messageHandlers).add(handler),
      }))

      return () => {
        set((state) => {
          const newHandlers = new Set(state.messageHandlers)
          newHandlers.delete(handler)
          return { messageHandlers: newHandlers }
        })
      }
    },
  }
})

// Auto-connect WebSocket when store is used in browser
if (typeof window !== 'undefined') {
  useWebSocketStore.getState().connect('ws://localhost:9000/ws')
}
