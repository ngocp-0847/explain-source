import { useEffect, useState, useRef } from 'react'

export const useSocket = () => {
  const [socket, setSocket] = useState<WebSocket | null>(null)
  const [isConnected, setIsConnected] = useState(false)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null)

  useEffect(() => {
    const connectWebSocket = () => {
      try {
        const newSocket = new WebSocket('ws://localhost:8080/ws')
        
        newSocket.onopen = () => {
          setIsConnected(true)
          console.log('Đã kết nối với Rust backend')
          if (reconnectTimeoutRef.current) {
            clearTimeout(reconnectTimeoutRef.current)
            reconnectTimeoutRef.current = null
          }
        }

        newSocket.onclose = () => {
          setIsConnected(false)
          console.log('Mất kết nối với Rust backend')
          
          // Tự động kết nối lại sau 3 giây
          if (reconnectTimeoutRef.current) {
            clearTimeout(reconnectTimeoutRef.current)
          }
          reconnectTimeoutRef.current = setTimeout(() => {
            console.log('Đang thử kết nối lại...')
            connectWebSocket()
          }, 3000)
        }

        newSocket.onerror = (error) => {
          console.error('Lỗi WebSocket:', error)
          setIsConnected(false)
        }

        setSocket(newSocket)
      } catch (error) {
        console.error('Lỗi tạo WebSocket:', error)
        setIsConnected(false)
      }
    }

    connectWebSocket()

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      if (socket) {
        socket.close()
      }
    }
  }, [])

  return { socket, isConnected }
}