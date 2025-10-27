# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

QA Chatbot MVP - A bilingual (Vietnamese UI) application that helps QA engineers understand business flow from source code. Built with Next.js (frontend) + Rust (backend) + WebSocket for real-time communication.

**Core Architecture**:
```
Frontend (Next.js/React) ←→ WebSocket ←→ Backend (Rust/Axum) ←→ Cursor Agent (simulated)
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

### Running the Full Stack
**Required**: Both frontend and backend must run simultaneously.

```bash
# Terminal 1: Start Rust backend
cd rust-backend && cargo run

# Terminal 2: Start Next.js frontend
npm run dev
```

Access points:
- Frontend: http://localhost:3010
- Backend API: http://localhost:8080
- WebSocket: ws://localhost:8080/ws

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
  - `cursor_agent.rs`: Cursor Agent integration (currently simulated)

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
4. Backend invokes Cursor Agent (currently simulated)
5. Backend streams results back via WebSocket
6. Frontend displays real-time analysis in chat interface

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

1. **Cursor Agent**: Currently simulated, not integrated with real Cursor Agent
2. **Persistence**: No database - tickets stored in-memory only
3. **Authentication**: No auth system
4. **Error Handling**: Basic error logging, no user-facing error UI
5. **Chat History**: Chat messages not persisted or shared across sessions

## Future Extension Points

The README.md mentions planned features:
- Real Cursor Agent integration
- Database for ticket/chat persistence
- Authentication system
- File upload for source code analysis
- Advanced code analysis types

When implementing these, consider:
- Database: Likely PostgreSQL or SQLite with async driver (sqlx, diesel-async)
- Auth: JWT tokens via Axum middleware
- File upload: Multipart form handling in Axum
- Cursor Agent: Replace `cursor_agent.rs` simulation with real API client
