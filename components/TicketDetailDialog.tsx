'use client'

import { useEffect } from 'react'
import { Ticket } from '@/types/ticket'
import { X, CodeIcon, Play, FileText, Square } from 'lucide-react'
import { LogViewer } from './LogViewer'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useUIStore } from '@/stores/uiStore'
import { useTicketStore } from '@/stores/ticketStore'

interface TicketDetailDialogProps {
  onStartAnalysis: (ticketId: string) => void
  onStopAnalysis: (ticketId: string) => void
}

export function TicketDetailDialog({ onStartAnalysis, onStopAnalysis }: TicketDetailDialogProps) {
  const { isOpen, selectedTicketId, onClose } = useUIStore(state => ({
    isOpen: state.isDetailModalOpen,
    selectedTicketId: state.selectedTicketIdForDetail,
    onClose: state.closeDetailModal,
  }))
  
  // Get live ticket data from ticketStore
  const tickets = useTicketStore(state => state.tickets)
  const loadTicketLogs = useTicketStore(state => state.loadTicketLogs)
  const ticket = tickets.find(t => t.id === selectedTicketId)

  // Load logs when dialog opens
  useEffect(() => {
    if (isOpen && selectedTicketId) {
      loadTicketLogs(selectedTicketId)
    }
  }, [isOpen, selectedTicketId, loadTicketLogs])

  if (!ticket || !isOpen) return null

  const getStatusBadge = () => {
    const variants = {
      todo: { variant: 'secondary' as const, label: 'Cần Làm' },
      'in-progress': { variant: 'default' as const, label: 'Đang Làm' },
      done: { variant: 'outline' as const, label: 'Hoàn Thành' },
    }
    const config = variants[ticket.status]
    return <Badge variant={config.variant}>{config.label}</Badge>
  }

  return (
    <>
      {isOpen && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-lg w-full max-w-6xl max-h-[90vh] flex flex-col shadow-2xl">
            {/* Header */}
            <div className="flex justify-between items-start p-6 border-b">
              <div className="flex-1">
                <h2 className="text-2xl font-bold text-gray-900 mb-2">{ticket.title}</h2>
                <div className="flex items-center gap-3">
                  {getStatusBadge()}
                  <span className="text-sm text-gray-500">
                    {ticket.createdAt.toLocaleDateString('vi-VN')}
                  </span>
                </div>
              </div>

              <Button variant="ghost" size="icon" onClick={onClose}>
                <X className="w-6 h-6" />
              </Button>
            </div>

            {/* Content - Scrollable */}
            <ScrollArea className="flex-1 overflow-hidden">
              <div className="p-6 space-y-6">
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
                      <code className="text-blue-700 font-mono text-sm">{ticket.codeContext}</code>
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
                        <Button disabled>
                          <Play className="w-4 h-4 mr-2" />
                          Đang Phân Tích...
                        </Button>
                        <Button
                          variant="destructive"
                          onClick={() => onStopAnalysis(ticket.id)}
                        >
                          <Square className="w-4 h-4 mr-2" />
                          Dừng
                        </Button>
                      </div>
                    ) : (
                      <Button onClick={() => onStartAnalysis(ticket.id)}>
                        <Play className="w-4 h-4 mr-2" />
                        Bắt Đầu Phân Tích
                      </Button>
                    )}
                  </div>

                  {/* Log Viewer */}
                  <div className="border border-gray-200 rounded-lg p-4 bg-gray-50">
                    <LogViewer logs={ticket.logs} isAnalyzing={ticket.isAnalyzing} />
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
            </ScrollArea>
          </div>
        </div>
      )}
    </>
  )
}
