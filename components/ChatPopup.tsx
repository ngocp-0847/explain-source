'use client'

import { useState } from 'react'
import { BotIcon, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ChatInterface } from './ChatInterface'

interface ChatPopupProps {
  isConnected: boolean
}

export function ChatPopup({ isConnected }: ChatPopupProps) {
  const [isOpen, setIsOpen] = useState(false)

  return (
    <>
      {/* Collapsed button */}
      {!isOpen && (
        <button
          onClick={() => setIsOpen(true)}
          className="fixed bottom-4 right-4 w-14 h-14 bg-blue-600 hover:bg-blue-700 text-white rounded-full shadow-lg flex items-center justify-center z-50 transition-colors"
          aria-label="Mở Chat Assistant"
        >
          <BotIcon className="w-6 h-6" />
        </button>
      )}

      {/* Expanded popup */}
      {isOpen && (
        <div className="fixed bottom-4 right-4 w-96 h-[450px] bg-white rounded-lg shadow-lg border border-gray-200 flex flex-col z-50">
          <div className="p-4 border-b border-gray-200 flex items-center justify-between">
            <h3 className="font-semibold text-gray-900 flex items-center">
              <BotIcon className="w-5 h-5 mr-2 text-blue-600" />
              Chat Assistant
            </h3>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsOpen(false)}
              className="h-8 w-8 p-0"
              aria-label="Đóng Chat Assistant"
            >
              <X className="w-4 h-4" />
            </Button>
          </div>
          <div className="flex-1 overflow-hidden">
            <ChatInterface isConnected={isConnected} hideHeader={true} />
          </div>
        </div>
      )}
    </>
  )
}

