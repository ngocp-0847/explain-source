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

### Phương pháp 1: Sử dụng Script Helper (Khuyến nghị)

#### 🚀 Setup và chạy một lần
```bash
# Cài đặt và khởi động ứng dụng
chmod +x setup.sh run.sh stop.sh
./setup.sh

# Script sẽ hỏi bạn có muốn chạy ngay không, chọn 'y' để khởi động
```

#### 🎯 Chạy ứng dụng
```bash
# Khởi động ứng dụng (tự động kiểm tra và restart nếu đang chạy)
./run.sh

# Dừng ứng dụng
./stop.sh
```

### Phương pháp 2: Sử dụng PM2 trực tiếp

```bash
# Khởi động
pm2 start ecosystem.config.js

# Xem trạng thái
pm2 status

# Xem logs
pm2 logs

# Dừng
pm2 stop ecosystem.config.js

# Khởi động lại
pm2 restart ecosystem.config.js
```

### Phương pháp 3: Chạy thủ công (không dùng PM2)

```bash
# 1. Cài đặt dependencies
npm install
cd rust-backend && cargo build && cd ..

# 2. Chạy ứng dụng (2 terminal riêng biệt)

# Terminal 1: Chạy Rust backend
cd rust-backend
cargo run

# Terminal 2: Chạy Next.js frontend
npm run dev
```

### Truy cập ứng dụng

- Frontend: http://localhost:3010
- Backend API: http://localhost:8080
- WebSocket: ws://localhost:8080/ws

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

## Scripts Helper có sẵn

Dự án cung cấp các script để quản lý dễ dàng:

| Script | Mô tả |
|--------|-------|
| `./setup.sh` | Cài đặt dependencies và thiết lập môi trường |
| `./run.sh` | Khởi động ứng dụng với PM2 (tự động restart nếu đang chạy) |
| `./stop.sh` | Dừng ứng dụng với PM2 |

## Quản lý ứng dụng với PM2

### Lệnh cơ bản
```bash
# Khởi động ứng dụng
pm2 start ecosystem.config.js

# Dừng ứng dụng
pm2 stop ecosystem.config.js

# Khởi động lại
pm2 restart ecosystem.config.js

# Xem logs
pm2 logs

# Xem trạng thái
pm2 status
```

### Quản lý từng service
```bash
# Xem logs frontend
pm2 logs qa-chatbot-frontend

# Xem logs backend
pm2 logs qa-chatbot-backend

# Khởi động lại frontend
pm2 restart qa-chatbot-frontend

# Khởi động lại backend
pm2 restart qa-chatbot-backend

# Dừng frontend
pm2 stop qa-chatbot-frontend

# Dừng backend
pm2 stop qa-chatbot-backend
```

### Lệnh nâng cao
```bash
# Monitoring real-time
pm2 monit

# Xem thông tin chi tiết
pm2 show qa-chatbot-frontend
pm2 show qa-chatbot-backend

# Xem metrics
pm2 list

# Xóa tất cả processes
pm2 delete all

# Save PM2 processes để tự động khởi động sau reboot
pm2 save
pm2 startup
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