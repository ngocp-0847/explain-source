'use client'

import { useEffect, useRef, useState } from 'react'
import { StructuredLog, LogMessageType } from '@/types/ticket'
import { Loader2, ChevronDown, ChevronRight } from 'lucide-react'

interface LogViewerProps {
  logs: StructuredLog[]
  isAnalyzing: boolean
}

export function LogViewer({ logs, isAnalyzing }: LogViewerProps) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const [autoScroll, setAutoScroll] = useState(true)

  useEffect(() => {
    if (autoScroll && scrollRef.current) {
      scrollRef.current.scrollIntoView({ behavior: 'smooth', block: 'end' })
    }
  }, [logs, autoScroll])

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const element = e.currentTarget
    const isAtBottom = Math.abs(
      element.scrollHeight - element.clientHeight - element.scrollTop
    ) < 50
    setAutoScroll(isAtBottom)
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-semibold text-gray-700">
          Nhật Ký Phân Tích {logs.length > 0 && `(${logs.length})`}
        </h3>
        {logs.length > 0 && (
          <button
            onClick={() => setAutoScroll(!autoScroll)}
            className="text-xs text-gray-500 hover:text-gray-700"
          >
            {autoScroll ? '🔄 Auto-scroll: ON' : '⏸️ Auto-scroll: OFF'}
          </button>
        )}
      </div>

      <div
        className="log-viewer flex-1 max-h-96 overflow-y-auto bg-gray-900 rounded-lg font-mono text-sm"
        onScroll={handleScroll}
      >
        <div className="p-4 space-y-2">
          {logs.length === 0 && !isAnalyzing && (
            <div className="text-gray-500 text-center py-8">
              Chưa có log nào. Click "Bắt Đầu Phân Tích" để bắt đầu.
            </div>
          )}

          {logs.map((log) => (
            <LogEntry key={log.id} log={log} />
          ))}

          {isAnalyzing && (
            <div className="flex items-center gap-2 text-blue-400 py-2">
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>Đang phân tích...</span>
            </div>
          )}

          <div ref={scrollRef} />
        </div>
      </div>
    </div>
  )
}

function LogEntry({ log }: { log: StructuredLog }) {
  const [expanded, setExpanded] = useState(false)

  const config = getLogConfig(log.messageType)

  const hasMetadata = log.metadata && Object.keys(log.metadata).length > 0

  return (
    <div className={`${config.bgColor} border-l-4 ${config.borderColor} p-3 rounded-r transition-all`}>
      <div className="flex items-start gap-2">
        <span className="text-lg flex-shrink-0">{config.icon}</span>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className={`text-xs font-medium ${config.textColor}`}>
              {config.label}
            </span>
            <span className="text-xs text-gray-500">
              {new Date(log.timestamp).toLocaleTimeString('vi-VN', {
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
              })}
            </span>
          </div>

          <div className="text-white break-words">{log.content}</div>

          {hasMetadata && (
            <div className="mt-2">
              <button
                onClick={() => setExpanded(!expanded)}
                className="flex items-center gap-1 text-xs text-gray-400 hover:text-gray-200"
              >
                {expanded ? (
                  <ChevronDown className="w-3 h-3" />
                ) : (
                  <ChevronRight className="w-3 h-3" />
                )}
                Metadata ({Object.keys(log.metadata!).length})
              </button>

              {expanded && (
                <div className="mt-2 text-xs text-gray-400 bg-gray-950 p-2 rounded">
                  <pre className="whitespace-pre-wrap break-all">
                    {JSON.stringify(log.metadata, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          )}

          {log.rawLog && expanded && (
            <details className="mt-2 text-xs">
              <summary className="text-gray-400 cursor-pointer hover:text-gray-200">
                Raw Log
              </summary>
              <pre className="mt-1 text-gray-500 whitespace-pre-wrap break-all">
                {log.rawLog}
              </pre>
            </details>
          )}
        </div>
      </div>
    </div>
  )
}

function getLogConfig(messageType: LogMessageType) {
  switch (messageType) {
    case 'tool_use':
      return {
        icon: '🔧',
        label: 'TOOL',
        bgColor: 'bg-blue-900/30',
        borderColor: 'border-blue-500',
        textColor: 'text-blue-400',
      }
    case 'assistant':
      return {
        icon: '💬',
        label: 'ASSISTANT',
        bgColor: 'bg-green-900/30',
        borderColor: 'border-green-500',
        textColor: 'text-green-400',
      }
    case 'error':
      return {
        icon: '❌',
        label: 'ERROR',
        bgColor: 'bg-red-900/30',
        borderColor: 'border-red-500',
        textColor: 'text-red-400',
      }
    case 'system':
      return {
        icon: 'ℹ️',
        label: 'SYSTEM',
        bgColor: 'bg-gray-800/30',
        borderColor: 'border-gray-500',
        textColor: 'text-gray-400',
      }
  }
}
