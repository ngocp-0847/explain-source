# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

QA Chatbot MVP - A bilingual (Vietnamese UI) application that helps QA engineers understand business flow from source code. Built with Next.js (frontend) + Rust (backend) + WebSocket for real-time communication.

**Core Architecture**:
```
Frontend (Next.js/React) ←→ WebSocket ←→ Backend (Rust/Axum) ←→ Code Analysis Agents (Claude Code, Gemini CLI, Cursor Agent)
```

## Initial Setup

### Mac M1/Apple Silicon Setup

This project was developed on Mac M1. Follow these steps for initial setup:

1. **Install Rust** (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Follow prompts, then restart terminal or run:
source $HOME/.cargo/env
```

2. **Install Node.js dependencies**:
```bash
npm install
```

3. **Initialize Rust backend**:
```bash
cd rust-backend
cargo build
cd ..
```

**M1-Specific Notes**:
- Rust toolchain should automatically detect ARM64 architecture
- If you encounter OpenSSL issues, install via Homebrew: `brew install openssl`
- Node.js should be ARM64 version for best performance

## Development Commands

### Frontend (Next.js)
```bash
# Development server on port 3010
npm run dev

# Production build
npm run build

# Production start
npm start

# Linting
npm run lint
```

### Backend (Rust)
```bash
# Development (from project root)
npm run rust:dev

# Or directly from rust-backend directory
cd rust-backend && cargo run

# Production build
npm run rust:build
# Or: cd rust-backend && cargo build --release
```

### Agent Configuration

The backend supports multiple code analysis agents configured via `.env` file.

#### Quick Setup (Recommended)

**First-time setup:**
```bash
# 1. Navigate to backend directory
cd rust-backend

# 2. Copy environment template
cp .env.example .env

# 3. Edit .env file with your preferred settings
nano .env  # or use your preferred editor

# 4. Run the backend (configuration auto-loaded)
cargo run
```

The `.env` file contains all configuration in one place - no need to remember or type environment variables!

#### Agent Selection

Edit `AGENT_TYPE` in `rust-backend/.env`:
```bash
# Choose your agent (default: claude)
AGENT_TYPE=claude  # Use Claude Code (default, recommended)
# or
AGENT_TYPE=gemini  # Use Gemini CLI
# or
AGENT_TYPE=cursor  # Use Cursor Agent
```

#### Available Configuration Options

All options can be set in `rust-backend/.env` file. See `.env.example` for full documentation.

**Claude Code Agent Configuration:**
- `CLAUDE_AGENT_PATH`: Path to claude executable (default: `"claude"`)
- `CLAUDE_AGENT_TIMEOUT`: Timeout in seconds (default: `300`)
- `CLAUDE_AGENT_MAX_RETRIES`: Maximum retry attempts (default: `2`)
- `CLAUDE_AGENT_WORKING_DIR`: Working directory for analysis (optional)
- `CLAUDE_AGENT_OUTPUT_FORMAT`: Output format (default: `stream-json`)
- `CLAUDE_API_KEY`: API key for Claude (optional)

**Gemini CLI Configuration:**
- `GEMINI_AGENT_PATH`: Path to gemini executable (default: `"gemini"`)
- `GEMINI_AGENT_TIMEOUT`: Timeout in seconds (default: `300`)
- `GEMINI_AGENT_MAX_RETRIES`: Maximum retry attempts (default: `2`)
- `GEMINI_AGENT_WORKING_DIR`: Working directory for analysis (optional)
- `GEMINI_AGENT_OUTPUT_FORMAT`: Output format (default: `stream-json`)
- `GEMINI_API_KEY`: API key for Gemini (optional)

**Cursor Agent Configuration:**
- `CURSOR_AGENT_PATH`: Path to cursor-agent executable (default: `"cursor-agent"`)
- `CURSOR_AGENT_TIMEOUT`: Timeout in seconds (default: `300`)
- `CURSOR_AGENT_MAX_RETRIES`: Maximum retry attempts (default: `2`)
- `CURSOR_AGENT_WORKING_DIR`: Working directory for analysis (optional)
- `CURSOR_AGENT_OUTPUT_FORMAT`: Output format (default: `stream-json`)
- `CURSOR_API_KEY`: API key for Cursor (optional)

#### Claude Code CLI Setup

Before using Claude Code agent, install the Claude CLI:
```bash
# Install Claude CLI globally
npm install -g @anthropic-ai/claude-cli

# Verify installation
claude --version

# Login to your Anthropic account (one-time setup)
claude login

# After successful login, backend can use Claude Code non-interactively
```

Reference: [Claude Code Headless Mode Documentation](https://code.claude.com/docs/en/headless)

#### Gemini CLI Setup

Before using Gemini CLI, complete Google OAuth authentication:
```bash
# 1. Install Gemini CLI
npm install -g @google/generative-ai-cli

# 2. Login interactively (one-time setup)
gemini

# 3. Complete OAuth in browser
# 4. After successful login, backend can use Gemini non-interactively
```

#### Advanced: Override Configuration

You can still override `.env` values with command-line environment variables:
```bash
# Temporarily use Gemini Agent (without editing .env)
AGENT_TYPE=gemini cargo run

# Temporarily use Cursor Agent
AGENT_TYPE=cursor cargo run

# Override timeout for Claude Code agent this run only
CLAUDE_AGENT_TIMEOUT=600 cargo run

# Override timeout for Gemini CLI
GEMINI_AGENT_TIMEOUT=600 cargo run
```

**Priority:** CLI env vars > `.env` file > default values

### Running the Full Stack
**Required**: Both frontend and backend must run simultaneously.

```bash
# Terminal 1: Start Rust backend (config auto-loaded from .env)
cd rust-backend
cargo run

# Terminal 2: Start Next.js frontend
npm run dev
```

**Note:** Agent selection and all configuration is controlled via `rust-backend/.env` file. No need to set environment variables manually!

Access points:
- Frontend: http://localhost:3010
- Backend API: http://localhost:9000
- WebSocket: ws://localhost:9000/ws

## Architecture Details

### Frontend Structure
- **Next.js 14** with App Router (`app/` directory)
- **React 18** with TypeScript
- **Drag & Drop**: `@dnd-kit` for Kanban board ticket management
- **WebSocket Client**: Native WebSocket with auto-reconnect (see `hooks/useSocket.ts`)
- **State Management**: React hooks (no external state library)

### Backend Structure
- **Rust**: Axum web framework with Tokio async runtime
- **WebSocket**: Native WebSocket support via Axum's `ws` extractor
- **Broadcast**: Tokio broadcast channel for pub/sub messaging
- **Components**:
  - `main.rs`: Server setup, routing, health check
  - `websocket_handler.rs`: WebSocket connection handling
  - `claude_agent.rs`: Claude Code Agent integration (headless mode)
  - `gemini_agent.rs`: Gemini CLI Agent integration
  - `cursor_agent.rs`: Cursor Agent integration
  - `agent_factory.rs`: Agent selection and initialization

### Real-Time Communication Flow

**Client → Server Messages**:
```json
{
  "type": "start-code-analysis",
  "ticketId": "string",
  "codeContext": "path/to/file.js",
  "question": "user question"
}
```

**Server → Client Messages**:
```json
{
  "message_type": "code-analysis-complete",
  "ticket_id": "string",
  "content": "analysis result",
  "timestamp": "ISO8601"
}

{
  "message_type": "cursor-agent-log",
  "content": "log message",
  "timestamp": "ISO8601"
}
```

### Key Interaction Pattern

1. User creates a ticket in "Todo" column
2. Drag ticket to "In Progress" → **Triggers WebSocket message** to backend
3. Backend receives `start-code-analysis` event
4. Backend invokes Code Analysis Agent (Claude Code by default, or Gemini/Cursor)
5. Agent analyzes code in the project directory
6. Backend streams analysis results back via WebSocket
7. Frontend displays real-time analysis in chat interface

### TypeScript Types

**Core Domain Models** (`types/ticket.ts`):
- `TicketStatus`: `'todo' | 'in-progress' | 'done'`
- `Ticket`: Main ticket entity with id, title, description, status, createdAt, codeContext, analysisResult
- `CodeAnalysis`: Analysis result container with ticket linkage

### State Management Pattern

**No Redux/Zustand** - Uses React's built-in state:
- `useState` for component-local state
- Props drilling for parent-child communication
- WebSocket for server-client state sync
- No global state store

### WebSocket Reliability

The `useSocket` hook implements:
- **Auto-reconnect**: 3-second delay after disconnect
- **Connection status**: Exposed via `isConnected` boolean
- **Cleanup**: Proper socket close on unmount
- **Error handling**: Console logging for debugging

### Component Architecture

**Page Components**:
- `app/page.tsx`: Main layout orchestrator, DnD context provider, WebSocket integration

**Feature Components**:
- `KanbanBoard`: Container for ticket columns
- `KanbanColumn`: Droppable column with SortableContext
- `KanbanCard`: Draggable ticket card
- `ChatInterface`: Real-time chat with bot for code analysis
- `TicketModal`: Create/edit ticket form

### Styling

- **Tailwind CSS**: Utility-first styling
- **Responsive**: Mobile-first with `lg:` breakpoints
- **Custom CSS**: Minimal, mostly in component classes
- **Icons**: Lucide React for consistent iconography

### Backend State Management

**Rust AppState**:
```rust
pub struct AppState {
    pub cursor_agent: Arc<CursorAgent>,
    pub broadcast_tx: broadcast::Sender<BroadcastMessage>,
}
```

- **Shared State**: Cloned Arc for thread-safe access across connections
- **Broadcast Channel**: 1000-message buffer for pub/sub
- **Per-Connection**: Each WebSocket gets own receiver from broadcast

## Testing Considerations

**No test suite currently exists**. When adding tests:
- Frontend: Consider React Testing Library for components
- Backend: Use `cargo test` for Rust unit tests
- Integration: Test WebSocket message flow end-to-end
- Mock: Cursor Agent should be mockable interface

## Known Limitations

1. **Code Analysis Agents**: Requires CLI installation (Claude Code recommended, Gemini CLI or Cursor Agent as alternatives)
2. **Authentication**: No auth system
3. **Error Handling**: Basic error logging, no user-facing error UI
4. **Multi-user**: Session data not shared across users
5. **Rate Limiting**: No rate limiting on agent API calls

## Future Extension Points

Planned features for enhancement:
- **Authentication system**: User accounts and session management
- **Rate limiting**: API rate limiting for agent calls
- **Advanced analysis**: Support for custom analysis prompts and templates
- **Multi-project**: Better project workspace management
- **Agent orchestration**: Automatic agent selection based on query type
- **Caching**: Cache analysis results for repeated queries

When implementing these, consider:
- Database: Currently using SQLite with sqlx async driver
- Auth: JWT tokens via Axum middleware
- Caching: Redis or in-memory cache for agent responses
- Agent API: Abstract agent interface allows easy addition of new providers
