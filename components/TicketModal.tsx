'use client'

import { useState, useEffect } from 'react'
import { Ticket } from '@/types/ticket'
import { XIcon } from 'lucide-react'

interface TicketModalProps {
  isOpen: boolean
  onClose: () => void
  onSubmit: (ticket: Omit<Ticket, 'id' | 'createdAt'>) => void
  ticket?: Ticket | null
}

export function TicketModal({ isOpen, onClose, onSubmit, ticket }: TicketModalProps) {
  const [formData, setFormData] = useState({
    title: '',
    description: '',
    codeContext: ''
  })

  useEffect(() => {
    if (ticket) {
      setFormData({
        title: ticket.title,
        description: ticket.description,
        codeContext: ticket.codeContext || ''
      })
    } else {
      setFormData({
        title: '',
        description: '',
        codeContext: ''
      })
    }
  }, [ticket])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.title.trim() || !formData.description.trim()) return

    onSubmit({
      title: formData.title,
      description: formData.description,
      status: 'todo',
      codeContext: formData.codeContext || undefined
    })
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-md mx-4">
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-900">
            {ticket ? 'Chỉnh sửa Ticket' : 'Tạo Ticket Mới'}
          </h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <XIcon className="w-6 h-6" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          <div>
            <label htmlFor="title" className="block text-sm font-medium text-gray-700 mb-1">
              Tiêu đề *
            </label>
            <input
              type="text"
              id="title"
              value={formData.title}
              onChange={(e) => setFormData(prev => ({ ...prev, title: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Ví dụ: Hiểu flow đăng nhập user"
              required
            />
          </div>

          <div>
            <label htmlFor="description" className="block text-sm font-medium text-gray-700 mb-1">
              Mô tả câu hỏi *
            </label>
            <textarea
              id="description"
              value={formData.description}
              onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Mô tả chi tiết câu hỏi về business flow..."
              required
            />
          </div>

          <div>
            <label htmlFor="codeContext" className="block text-sm font-medium text-gray-700 mb-1">
              Code Context (tùy chọn)
            </label>
            <input
              type="text"
              id="codeContext"
              value={formData.codeContext}
              onChange={(e) => setFormData(prev => ({ ...prev, codeContext: e.target.value }))}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Ví dụ: auth/login.js, api/payment.js"
            />
            <p className="text-xs text-gray-500 mt-1">
              Đường dẫn file hoặc module code liên quan
            </p>
          </div>

          <div className="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
            >
              Hủy
            </button>
            <button
              type="submit"
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              {ticket ? 'Cập nhật' : 'Tạo Ticket'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}