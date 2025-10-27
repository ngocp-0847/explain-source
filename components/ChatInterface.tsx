'use client'

import { useState, useEffect, useRef } from 'react'
import { Ticket } from '@/types/ticket'
import { SendIcon, BotIcon, UserIcon } from 'lucide-react'

interface ChatInterfaceProps {
  selectedTicket: Ticket | null
  isConnected: boolean
}

interface ChatMessage {
  id: string
  type: 'user' | 'bot'
  content: string
  timestamp: Date
}

export function ChatInterface({ selectedTicket, isConnected }: ChatInterfaceProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [inputMessage, setInputMessage] = useState('')
  const [isTyping, setIsTyping] = useState(false)
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
      setMessages([
        {
          id: '1',
          type: 'bot',
          content: `Tôi sẽ giúp bạn phân tích ticket: "${selectedTicket.title}"`,
          timestamp: new Date()
        }
      ])
    }
  }, [selectedTicket])

  const handleSendMessage = async () => {
    if (!inputMessage.trim() || !isConnected) return

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      type: 'user',
      content: inputMessage,
      timestamp: new Date()
    }

    setMessages(prev => [...prev, userMessage])
    setInputMessage('')
    setIsTyping(true)

    // Simulate bot response (in real app, this would call the Rust backend)
    setTimeout(() => {
      const botMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        type: 'bot',
        content: `Tôi đang phân tích code trong context "${selectedTicket?.codeContext || 'N/A'}" để trả lời câu hỏi của bạn...`,
        timestamp: new Date()
      }
      setMessages(prev => [...prev, botMessage])
      setIsTyping(false)
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

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
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

      <div className="p-4 border-t border-gray-200">
        <div className="flex space-x-2">
          <input
            type="text"
            value={inputMessage}
            onChange={(e) => setInputMessage(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Nhập câu hỏi về code..."
            className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            disabled={!isConnected}
          />
          <button
            onClick={handleSendMessage}
            disabled={!inputMessage.trim() || !isConnected}
            className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <SendIcon className="w-4 h-4" />
          </button>
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