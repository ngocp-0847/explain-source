# Demo QA Chatbot MVP

## Cách chạy demo

### Phương pháp 1: Chạy trực tiếp
```bash
./start.sh
```

### Phương pháp 2: Docker
```bash
docker-compose up --build
```

## Demo Flow

1. **Truy cập**: http://localhost:3010
2. **Tạo ticket mới**: Click "Tạo Ticket Mới"
   - Tiêu đề: "Hiểu flow đăng nhập user"
   - Mô tả: "Cần hiểu cách hệ thống xử lý đăng nhập user"
   - Code Context: "auth/login.js"
3. **Kéo ticket**: Drag ticket từ "Cần Làm" sang "Đang Xử Lý"
4. **Xem phân tích**: Hệ thống sẽ tự động trigger Cursor Agent
5. **Chat thêm**: Sử dụng chat interface để hỏi thêm

## Tính năng đã implement

✅ **Kanban Board**: Drag & drop tickets giữa các cột
✅ **WebSocket**: Real-time communication
✅ **Cursor Agent Integration**: Phân tích code tự động
✅ **Chat Interface**: Chat với AI assistant
✅ **Ticket Management**: Tạo, chỉnh sửa tickets
✅ **Responsive UI**: Giao diện đẹp với Tailwind CSS
✅ **TypeScript**: Type safety
✅ **Rust Backend**: High performance WebSocket server

## Cấu trúc dữ liệu

### Ticket
```typescript
interface Ticket {
  id: string
  title: string
  description: string
  status: 'todo' | 'in-progress' | 'done'
  createdAt: Date
  codeContext?: string
  analysisResult?: string
}
```

### WebSocket Events
- `start-code-analysis`: Trigger phân tích code
- `code-analysis`: Kết quả phân tích
- `cursor-agent-log`: Log real-time từ Cursor Agent

## Mở rộng trong tương lai

1. **Database**: PostgreSQL để lưu trữ tickets
2. **Authentication**: JWT-based auth
3. **File Upload**: Upload source code files
4. **Advanced AI**: Tích hợp GPT-4 hoặc Claude
5. **Team Collaboration**: Multi-user support
6. **Analytics**: Tracking và reporting