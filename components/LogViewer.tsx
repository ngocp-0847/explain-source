'use client'

import { useEffect, useRef, useState } from 'react'
import { StructuredLog, LogMessageType } from '@/types/ticket'
import { Loader2, ChevronDown, ChevronRight } from 'lucide-react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

// Utility function để parse content JSON và extract type
function parseLogContent(content: string): { 
  parsedContent: any | null, 
  contentType: string | null,
  isJson: boolean 
} {
  try {
    const parsed = JSON.parse(content)
    return {
      parsedContent: parsed,
      contentType: parsed?.type || null,
      isJson: true
    }
  } catch {
    return {
      parsedContent: null,
      contentType: null,
      isJson: false
    }
  }
}

// Function để merge assistant logs thành markdown content
function mergeAssistantLogs(logs: StructuredLog[]): string {
  const assistantLogs = logs
    .filter(log => {
      const { contentType } = parseLogContent(log.content)
      return log.messageType === 'assistant' || 
             log.messageType === 'result' ||  // Include result logs
             contentType === 'assistant' ||
             contentType === 'result'         // Include result content type
    })
    .sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime())

  const markdownParts: string[] = []

  for (const log of assistantLogs) {
    const { parsedContent } = parseLogContent(log.content)
    
    if (parsedContent?.content && Array.isArray(parsedContent.content)) {
      // Extract text từ content array
      const textParts = parsedContent.content
        .filter((item: any) => item.type === 'text')
        .map((item: any) => item.text)
        .join('\n\n')
      
      if (textParts.trim()) {
        markdownParts.push(textParts)
      }
    } else if (parsedContent?.result && typeof parsedContent.result === 'string') {
      // Handle result field from completion messages
      markdownParts.push(parsedContent.result)
    } else if (parsedContent?.message) {
      // Fallback cho structure khác
      markdownParts.push(parsedContent.message)
    } else if (typeof parsedContent === 'string') {
      // Plain text content
      markdownParts.push(parsedContent)
    } else if (typeof log.content === 'string' && !parsedContent) {
      // If parsing failed, use raw content
      markdownParts.push(log.content)
    }
  }

  return markdownParts.join('\n\n')
}

// Function để tạo summary text từ parsed content
function createLogSummary(content: string, parsedContent: any | null, contentType: string | null): string {
  if (!parsedContent || !contentType) {
    return content.length > 100 ? content.substring(0, 100) + '...' : content
  }

  switch (contentType) {
    case 'tool_call':
      const toolName = parsedContent.tool_call?.updateTodosToolCall ? 'updateTodos' : 
                      parsedContent.tool_call ? Object.keys(parsedContent.tool_call)[0] : 'unknown'
      const subtype = parsedContent.subtype || 'unknown'
      return `🔧 Tool Call: ${toolName} (${subtype})`
    
    case 'assistant':
      return `💬 Assistant: ${parsedContent.content?.substring(0, 50) || 'Message'}...`
    
    case 'error':
      return `❌ Error: ${parsedContent.message || parsedContent.error || 'Unknown error'}`
    
    default:
      return content.length > 100 ? content.substring(0, 100) + '...' : content
  }
}

// Simple JSON syntax highlighter
function highlightJson(jsonString: string): string {
  return jsonString
    .replace(/"([^"]+)":/g, '<span class="json-key">"$1":</span>')
    .replace(/: "([^"]+)"/g, ': <span class="json-string">"$1"</span>')
    .replace(/: (\d+)/g, ': <span class="json-number">$1</span>')
    .replace(/: (true|false)/g, ': <span class="json-boolean">$1</span>')
    .replace(/: null/g, ': <span class="json-null">null</span>')
}

interface LogViewerProps {
  logs: StructuredLog[]
  isAnalyzing: boolean
}

export function LogViewer({ logs = [], isAnalyzing }: LogViewerProps) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const [autoScroll, setAutoScroll] = useState(true)
  const [isLogsExpanded, setIsLogsExpanded] = useState(true)

  useEffect(() => {
    if (autoScroll && scrollRef.current) {
      scrollRef.current.scrollIntoView({ behavior: 'smooth', block: 'end' })
    }
  }, [logs, autoScroll])

  // Check if analysis is completed (has result log)
  const isAnalysisCompleted = logs.some(log => {
    const { contentType } = parseLogContent(log.content)
    return log.messageType === 'result' || contentType === 'result'
  })

  // Auto-collapse logs when analysis is completed
  useEffect(() => {
    if (isAnalysisCompleted) {
      setIsLogsExpanded(false)
    }
  }, [isAnalysisCompleted])

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const element = e.currentTarget
    const isAtBottom = Math.abs(
      element.scrollHeight - element.clientHeight - element.scrollTop
    ) < 50
    setAutoScroll(isAtBottom)
  }

  // Get merged markdown content from assistant logs
  const markdownContent = isAnalysisCompleted ? mergeAssistantLogs(logs) : ''

  return (
    <div className="flex flex-col h-full">
      {/* Markdown Preview Section */}
      {isAnalysisCompleted && markdownContent && (
        <MarkdownPreview content={markdownContent} />
      )}

      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-semibold text-gray-700">
            Nhật Ký Phân Tích {logs.length > 0 && `(${logs.length})`}
          </h3>
          {logs.length > 0 && (
            <button
              onClick={() => setIsLogsExpanded(!isLogsExpanded)}
              className="text-gray-500 hover:text-gray-700 transition-colors"
            >
              {isLogsExpanded ? (
                <ChevronDown className="w-4 h-4" />
              ) : (
                <ChevronRight className="w-4 h-4" />
              )}
            </button>
          )}
        </div>
        {logs.length > 0 && (
          <button
            onClick={() => setAutoScroll(!autoScroll)}
            className="text-xs text-gray-500 hover:text-gray-700"
          >
            {autoScroll ? '🔄 Auto-scroll: ON' : '⏸️ Auto-scroll: OFF'}
          </button>
        )}
      </div>

      {isLogsExpanded && (
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
      )}
    </div>
  )
}

function MarkdownPreview({ content }: { content: string }) {
  const [isExpanded, setIsExpanded] = useState(true)

  return (
    <div className="bg-white border border-gray-200 rounded-lg shadow-sm mb-4">
      <div className="flex items-center justify-between p-3 border-b border-gray-200">
        <h4 className="text-sm font-semibold text-gray-800 flex items-center gap-2">
          📝 Kết Quả Phân Tích
        </h4>
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="text-gray-500 hover:text-gray-700 transition-colors"
        >
          {isExpanded ? (
            <ChevronDown className="w-4 h-4" />
          ) : (
            <ChevronRight className="w-4 h-4" />
          )}
        </button>
      </div>
      
      {isExpanded && (
        <div className="p-4 max-h-96 overflow-y-auto markdown-preview">
          <div className="prose prose-sm max-w-none">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {content}
            </ReactMarkdown>
          </div>
        </div>
      )}
    </div>
  )
}

function LogEntry({ log }: { log: StructuredLog }) {
  const [expanded, setExpanded] = useState(false)
  const [jsonExpanded, setJsonExpanded] = useState(false)

  // Parse content để lấy type và summary
  const { parsedContent, contentType, isJson } = parseLogContent(log.content)
  const summary = createLogSummary(log.content, parsedContent, contentType)
  
  // Sử dụng contentType thay vì messageType để determine config
  const config = getLogConfig(contentType || log.messageType)

  const hasMetadata = log.metadata && Object.keys(log.metadata).length > 0

  // Special styling for completion result
  const isCompletionResult = contentType === 'result' || log.messageType === 'result'
  
  return (
    <div className={`${config.bgColor} border-l-4 ${config.borderColor} p-3 rounded-r transition-all ${
      isCompletionResult ? 'animate-pulse border-l-8 shadow-lg' : ''
    }`}>
      <div className="flex items-start gap-2">
        <span className={`text-lg flex-shrink-0 ${isCompletionResult ? 'animate-bounce' : ''}`}>
          {config.icon}
        </span>

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

          {/* Plain text summary */}
          <div className={`text-white break-words mb-2 ${isCompletionResult ? 'font-bold text-lg' : ''}`}>
            {summary}
          </div>
          
          {/* Special completion message */}
          {isCompletionResult && (
            <div className="mt-2 p-2 bg-emerald-800/20 rounded border border-emerald-500/30">
              <div className="text-emerald-300 text-sm font-medium">
                🎉 Phân tích đã hoàn tất thành công! Kết quả đã được lưu.
              </div>
            </div>
          )}

          {/* JSON Details Toggle */}
          {isJson && (
            <div className="mt-2">
              <button
                onClick={() => setJsonExpanded(!jsonExpanded)}
                className="flex items-center gap-1 text-xs text-gray-400 hover:text-gray-200"
              >
                {jsonExpanded ? (
                  <ChevronDown className="w-3 h-3" />
                ) : (
                  <ChevronRight className="w-3 h-3" />
                )}
                View JSON Details
              </button>

              {jsonExpanded && (
                <div className="mt-2 text-xs text-gray-400 bg-gray-950 p-3 rounded">
                  <pre 
                    className="whitespace-pre-wrap break-all json-highlight"
                    dangerouslySetInnerHTML={{
                      __html: highlightJson(JSON.stringify(parsedContent, null, 2))
                    }}
                  />
                </div>
              )}
            </div>
          )}

          {/* Metadata Section */}
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

          {/* Raw Log Section */}
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

function getLogConfig(messageType: string | LogMessageType) {
  switch (messageType) {
    case 'tool_call':
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
    case 'result':
      return {
        icon: '✅',
        label: 'COMPLETED',
        bgColor: 'bg-emerald-900/30',
        borderColor: 'border-emerald-500',
        textColor: 'text-emerald-400',
      }
    case 'system':
    default:
      // Fallback cho messageType không hợp lệ
      return {
        icon: 'ℹ️',
        label: 'SYSTEM',
        bgColor: 'bg-gray-800/30',
        borderColor: 'border-gray-500',
        textColor: 'text-gray-400',
      }
  }
}
