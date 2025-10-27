# QA Chatbot MVP - Code Flow Analysis

Sản phẩm MVP chatbot giúp QA hiểu business flow từ source code sử dụng Next.js + Rust.

## Tính năng chính

- 📋 **Kanban Board**: Quản lý ticket câu hỏi QA
- 🤖 **AI Chatbot**: Tích hợp Cursor Agent để phân tích code
- 🔄 **Real-time**: WebSocket để cập nhật real-time
- 📊 **Code Analysis**: Phân tích business flow từ source code

## Kiến trúc hệ thống

```
Frontend (Next.js) ←→ WebSocket ←→ Backend (Rust) ←→ Cursor Agent
```

## Cài đặt và chạy

### 1. Cài đặt dependencies

```bash
# Frontend
npm install

# Backend
cd rust-backend
cargo build
```

### 2. Chạy ứng dụng

```bash
# Terminal 1: Chạy Rust backend
cd rust-backend
cargo run

# Terminal 2: Chạy Next.js frontend
npm run dev
```

### 3. Truy cập ứng dụng

- Frontend: http://localhost:3010
- Backend API: http://localhost:8080

## Cách sử dụng

1. **Tạo Ticket**: Click "Tạo Ticket Mới" để tạo câu hỏi QA
2. **Drag & Drop**: Kéo ticket vào cột "Đang Xử Lý" để trigger phân tích
3. **Chat**: Sử dụng chat interface để hỏi thêm về code
4. **Real-time**: Xem kết quả phân tích real-time qua WebSocket

## Cấu trúc dự án

```
├── app/                    # Next.js app directory
│   ├── components/         # React components
│   ├── hooks/             # Custom hooks
│   └── types/             # TypeScript types
├── rust-backend/          # Rust backend
│   ├── src/
│   │   ├── main.rs        # Main server
│   │   ├── cursor_agent.rs # Cursor Agent integration
│   │   └── websocket_handler.rs # WebSocket handling
│   └── Cargo.toml
└── README.md
```

## API Endpoints

- `GET /` - Health check
- `WS /ws` - WebSocket connection

## WebSocket Events

### Client → Server
- `start-code-analysis`: Bắt đầu phân tích code
- `ping`: Ping connection

### Server → Client  
- `code-analysis`: Kết quả phân tích code
- `cursor-agent-log`: Log từ Cursor Agent
- `pong`: Pong response

## Công nghệ sử dụng

### Frontend
- Next.js 14
- TypeScript
- Tailwind CSS
- @dnd-kit (Drag & Drop)
- Socket.io-client
- Lucide React (Icons)

### Backend
- Rust
- Axum (Web framework)
- Tokio (Async runtime)
- WebSocket support
- Serde (Serialization)

## Phát triển thêm

1. **Tích hợp Cursor Agent thực**: Thay thế simulation bằng Cursor Agent thực
2. **Database**: Thêm database để lưu trữ tickets và lịch sử chat
3. **Authentication**: Thêm hệ thống đăng nhập
4. **File Upload**: Upload source code files để phân tích
5. **Advanced Analysis**: Thêm các loại phân tích code khác

## License

MIT