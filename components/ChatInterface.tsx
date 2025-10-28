'use client'

import { useState, useEffect, useRef } from 'react'
import { useChatStore } from '@/stores/chatStore'
import { SendIcon, BotIcon, UserIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useUIStore } from '@/stores/uiStore'
import { useTicketStore } from '@/stores/ticketStore'

interface ChatInterfaceProps {
  isConnected: boolean
}

export function ChatInterface({ isConnected }: ChatInterfaceProps) {
  const messages = useChatStore(state => state.messages)
  const addMessage = useChatStore(state => state.addMessage)
  const clearMessages = useChatStore(state => state.clearMessages)
  const isTyping = useChatStore(state => state.isTyping)
  const setTyping = useChatStore(state => state.setTyping)

  const [inputMessage, setInputMessage] = useState('')
  const selectedTicketId = useUIStore(state => state.selectedTicketIdForDetail)
  const tickets = useTicketStore(state => state.tickets)
  const selectedTicket = tickets.find(t => t.id === selectedTicketId)

  const messagesEndRef = useRef<HTMLDivElement>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  useEffect(() => {
    if (selectedTicket) {
      // Reset messages when ticket changes
      clearMessages()
      addMessage({
        id: crypto.randomUUID(),
        type: 'bot',
        content: `Tôi sẽ giúp bạn phân tích ticket: "${selectedTicket.title}"`,
        timestamp: new Date()
      })
    }
  }, [selectedTicket, clearMessages, addMessage])

  const handleSendMessage = async () => {
    if (!inputMessage.trim() || !isConnected) return

    const userMessage = {
      id: crypto.randomUUID(),
      type: 'user' as const,
      content: inputMessage,
      timestamp: new Date()
    }

    addMessage(userMessage)
    setInputMessage('')
    setTyping(true)

    // Simulate bot response
    setTimeout(() => {
      const botMessage = {
        id: crypto.randomUUID(),
        type: 'bot' as const,
        content: `Tôi đang phân tích code trong context "${selectedTicket?.codeContext || 'N/A'}" để trả lời câu hỏi của bạn...`,
        timestamp: new Date()
      }
      addMessage(botMessage)
      setTyping(false)
    }, 2000)
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSendMessage()
    }
  }

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 h-[600px] flex flex-col">
      <div className="p-4 border-b border-gray-200">
        <h3 className="font-semibold text-gray-900 flex items-center">
          <BotIcon className="w-5 h-5 mr-2 text-blue-600" />
          Chat Assistant
        </h3>
        {selectedTicket && (
          <p className="text-sm text-gray-600 mt-1">
            Đang phân tích: {selectedTicket.title}
          </p>
        )}
      </div>

      <ScrollArea className="flex-1 p-4">
        <div className="space-y-4">
          {messages.map(message => (
            <div
              key={message.id}
              className={`flex ${message.type === 'user' ? 'justify-end' : 'justify-start'}`}
            >
              <div className={`flex items-start space-x-2 max-w-xs ${
                message.type === 'user' ? 'flex-row-reverse space-x-reverse' : ''
              }`}>
                <div className={`w-8 h-8 rounded-full flex items-center justify-center ${
                  message.type === 'user' ? 'bg-blue-500' : 'bg-gray-200'
                }`}>
                  {message.type === 'user' ? (
                    <UserIcon className="w-4 h-4 text-white" />
                  ) : (
                    <BotIcon className="w-4 h-4 text-gray-600" />
                  )}
                </div>
                <div className={`chat-message ${message.type}`}>
                  <p className="text-sm">{message.content}</p>
                  <p className="text-xs opacity-70 mt-1">
                    {message.timestamp.toLocaleTimeString('vi-VN')}
                  </p>
                </div>
              </div>
            </div>
          ))}

          {isTyping && (
            <div className="flex justify-start">
              <div className="flex items-start space-x-2">
                <div className="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center">
                  <BotIcon className="w-4 h-4 text-gray-600" />
                </div>
                <div className="chat-message bot">
                  <div className="flex space-x-1">
                    <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce"></div>
                    <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0.1s' }}></div>
                    <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }}></div>
                  </div>
                </div>
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>
      </ScrollArea>

      <div className="p-4 border-t border-gray-200">
        <div className="flex space-x-2">
          <Input
            type="text"
            value={inputMessage}
            onChange={(e) => setInputMessage(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Nhập câu hỏi về code..."
            disabled={!isConnected}
          />
          <Button
            onClick={handleSendMessage}
            disabled={!inputMessage.trim() || !isConnected}
          >
            <SendIcon className="w-4 h-4" />
          </Button>
        </div>
        {!isConnected && (
          <p className="text-xs text-red-500 mt-2">
            Mất kết nối với server. Vui lòng thử lại.
          </p>
        )}
      </div>
    </div>
  )
}