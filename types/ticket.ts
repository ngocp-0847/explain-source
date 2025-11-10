export type TicketStatus = 'todo' | 'in-progress' | 'done'

export type TicketMode = 'plan' | 'ask' | 'edit'

export type LogMessageType = 'tool_use' | 'assistant' | 'error' | 'system' | 'result'

export interface Project {
  id: string
  name: string
  description?: string
  directoryPath: string
  createdAt: string
  updatedAt: string
}

export interface StructuredLog {
  id: string
  ticketId: string
  messageType: LogMessageType
  content: string
  rawLog?: string
  metadata?: Record<string, string>
  timestamp: string
}

export interface Ticket {
  id: string
  projectId: string
  title: string
  description: string
  status: TicketStatus
  createdAt: Date
  updatedAt?: Date
  codeContext?: string
  analysisResult?: string
  isAnalyzing: boolean
  logs: StructuredLog[]
  mode: TicketMode
  planContent?: string
  planCreatedAt?: Date
  requiredApprovals: number
}

export interface CodeAnalysis {
  ticketId: string
  codeContext: string
  question: string
  result: string
  logs: StructuredLog[]
  timestamp: Date
}

export interface AnalysisSession {
  id: string
  ticketId: string
  startedAt: Date
  completedAt?: Date
  status: 'running' | 'completed' | 'failed'
  errorMessage?: string
}

// WebSocket message types
export interface WebSocketMessage {
  message_type: string
  [key: string]: any
}

// Raw log structure từ backend (snake_case)
export interface RawStructuredLog {
  id: string
  ticket_id: string
  message_type: LogMessageType
  content: string
  raw_log?: string
  metadata?: Record<string, string>
  timestamp: string
}

// Paginated logs response từ backend
export interface PaginatedLogsResponse {
  logs: RawStructuredLog[]
  total: number
  has_more: boolean
}

export interface StructuredLogMessage extends WebSocketMessage {
  message_type: 'structured-log'
  log: RawStructuredLog
}

export interface CodeAnalysisCompleteMessage extends WebSocketMessage {
  message_type: 'code-analysis-complete'
  ticket_id: string
  content: string
  timestamp: string
}

export interface CodeAnalysisErrorMessage extends WebSocketMessage {
  message_type: 'code-analysis-error'
  ticket_id: string
  error: string
  timestamp: string
}

// Type guard để validate LogMessageType
export function isValidLogMessageType(type: string): type is LogMessageType {
  return ['tool_use', 'assistant', 'error', 'system', 'result'].includes(type)
}
