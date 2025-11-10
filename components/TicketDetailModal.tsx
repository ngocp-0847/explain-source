'use client'

import { Ticket } from '@/types/ticket'
import { X, CodeIcon, Play, FileText, Square } from 'lucide-react'
import { LogViewer } from './LogViewer'
// import ReactMarkdown from 'react-markdown' // Removed for now, will display as pre-formatted text

interface TicketDetailModalProps {
  ticket: Ticket | null
  isOpen: boolean
  onClose: () => void
  onStartAnalysis: (ticketId: string) => void
  onStopAnalysis: (ticketId: string) => void
}

export function TicketDetailModal({
  ticket,
  isOpen,
  onClose,
  onStartAnalysis,
  onStopAnalysis,
}: TicketDetailModalProps) {
  if (!isOpen || !ticket) return null

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg w-full max-w-6xl max-h-[90vh] flex flex-col shadow-2xl">
        {/* Header */}
        <div className="flex justify-between items-start p-6 border-b">
          <div className="flex-1">
            <h2 className="text-2xl font-bold text-gray-900 mb-2">
              {ticket.title}
            </h2>
            <div className="flex items-center gap-3">
              <span
                className={`px-3 py-1 rounded-full text-xs font-medium ${
                  ticket.status === 'todo'
                    ? 'bg-gray-100 text-gray-700'
                    : ticket.status === 'in-progress'
                    ? 'bg-blue-100 text-blue-700'
                    : 'bg-green-100 text-green-700'
                }`}
              >
                {ticket.status === 'todo'
                  ? 'Cần Làm'
                  : ticket.status === 'in-progress'
                  ? 'Đang Làm'
                  : 'Hoàn Thành'}
              </span>
              <span className="text-sm text-gray-500">
                {ticket.createdAt.toLocaleDateString('vi-VN')}
              </span>
            </div>
          </div>

          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Content - Scrollable */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {/* Description */}
          <section>
            <div className="flex items-center gap-2 mb-3">
              <FileText className="w-5 h-5 text-gray-600" />
              <h3 className="font-semibold text-gray-700">Mô tả</h3>
            </div>
            <p className="text-gray-600 leading-relaxed">{ticket.description}</p>
          </section>

          {/* Code Context */}
          {ticket.codeContext && (
            <section>
              <div className="flex items-center gap-2 mb-3">
                <CodeIcon className="w-5 h-5 text-gray-600" />
                <h3 className="font-semibold text-gray-700">Code Context</h3>
              </div>
              <div className="flex items-center gap-2 px-4 py-2 bg-blue-50 border border-blue-200 rounded-lg">
                <CodeIcon className="w-4 h-4 text-blue-600" />
                <code className="text-blue-700 font-mono text-sm">
                  {ticket.codeContext}
                </code>
              </div>
            </section>
          )}

          {/* Analysis Controls */}
          <section>
            <div className="flex justify-between items-center mb-3">
              <div className="flex items-center gap-2">
                <Play className="w-5 h-5 text-gray-600" />
                <h3 className="font-semibold text-gray-700">Phân Tích Code</h3>
              </div>

              {ticket.isAnalyzing ? (
                <div className="flex gap-2">
                  <button
                    disabled
                    className="flex items-center gap-2 px-4 py-2 rounded-lg font-medium bg-gray-400 text-white cursor-not-allowed"
                  >
                    <Play className="w-4 h-4" />
                    Đang Phân Tích...
                  </button>
                  <button
                    onClick={() => onStopAnalysis(ticket.id)}
                    className="flex items-center gap-2 px-4 py-2 rounded-lg font-medium bg-red-600 text-white hover:bg-red-700 hover:shadow-lg transition-all"
                  >
                    <Square className="w-4 h-4" />
                    Dừng
                  </button>
                </div>
              ) : (
                <button
                  onClick={() => onStartAnalysis(ticket.id)}
                  className="flex items-center gap-2 px-4 py-2 rounded-lg font-medium bg-blue-600 text-white hover:bg-blue-700 hover:shadow-lg transition-all"
                >
                  <Play className="w-4 h-4" />
                  Bắt Đầu Phân Tích
                </button>
              )}
            </div>

            {/* Log Viewer */}
            <div className="border border-gray-200 rounded-lg p-4 bg-gray-50">
              <LogViewer logs={ticket.logs} isAnalyzing={ticket.isAnalyzing} ticketId={ticket.id} />
            </div>
          </section>

          {/* Analysis Result */}
          {ticket.analysisResult && (
            <section>
              <div className="flex items-center gap-2 mb-3">
                <FileText className="w-5 h-5 text-green-600" />
                <h3 className="font-semibold text-gray-700">Kết Quả Phân Tích</h3>
              </div>
              <div className="bg-green-50 border border-green-200 rounded-lg p-6">
                <div className="whitespace-pre-wrap text-sm text-green-800 leading-relaxed font-mono">
                  {ticket.analysisResult}
                </div>
              </div>
            </section>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 p-6 border-t bg-gray-50">
          <button
            onClick={onClose}
            className="px-6 py-2 border border-gray-300 rounded-lg text-gray-700 font-medium hover:bg-gray-100 transition-colors"
          >
            Đóng
          </button>
        </div>
      </div>
    </div>
  )
}
