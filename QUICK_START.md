# 🚀 Quick Start - QA Chatbot MVP

Hướng dẫn nhanh để bắt đầu với dự án QA Chatbot MVP sử dụng PM2.

## ⚡️ Bắt đầu nhanh

### Cách 1: Setup và chạy một lần (Khuyến nghị)
```bash
chmod +x setup.sh run.sh stop.sh
./setup.sh
```
Script sẽ hỏi bạn có muốn chạy ngay không → chọn `y`

### Cách 2: Setup riêng, sau đó chạy
```bash
# Bước 1: Cài đặt
./setup.sh

# Bước 2: Khởi động
./run.sh

# Bước 3: Dừng (khi không dùng)
./stop.sh
```

## 📋 Các Scripts có sẵn

| Script | Mô tả |
|--------|-------|
| `./setup.sh` | Cài đặt dependencies và thiết lập môi trường |
| `./run.sh` | Khởi động ứng dụng (tự động restart nếu đang chạy) |
| `./stop.sh` | Dừng ứng dụng |

## 🎯 Sau khi chạy

Truy cập:
- Frontend: http://localhost:3010
- Backend: http://localhost:8080
- WebSocket: ws://localhost:8080/ws

Bạn sẽ thấy 2 processes trong PM2:
- `qa-chatbot-frontend` (Next.js)
- `qa-chatbot-backend` (Rust)

## 📋 Các lệnh hữu ích

### Xem logs
```bash
# Xem tất cả logs
pm2 logs

# Chỉ xem logs frontend
pm2 logs qa-chatbot-frontend

# Chỉ xem logs backend
pm2 logs qa-chatbot-backend

# Theo dõi logs real-time (Ctrl+C để thoát)
pm2 logs --lines 50
```

### Quản lý ứng dụng
```bash
# Dừng ứng dụng
pm2 stop ecosystem.config.js

# Khởi động lại
pm2 restart ecosystem.config.js

# Dừng tất cả
pm2 stop all
```

### Quản lý từng service
```bash
# Restart frontend
pm2 restart qa-chatbot-frontend

# Restart backend
pm2 restart qa-chatbot-backend

# Stop frontend
pm2 stop qa-chatbot-frontend

# Stop backend
pm2 stop qa-chatbot-backend
```

### Monitoring
```bash
# Xem dashboard monitoring
pm2 monit

# Xem thông tin chi tiết
pm2 show qa-chatbot-frontend
pm2 show qa-chatbot-backend
```

### Tự động khởi động sau khi reboot
```bash
# Lưu danh sách processes hiện tại
pm2 save

# Cấu hình tự động start sau reboot
pm2 startup

# Làm theo hướng dẫn mà pm2 đưa ra
```

## 🔧 Xử lý sự cố

### Frontend không chạy
```bash
# Kiểm tra logs
pm2 logs qa-chatbot-frontend

# Kiểm tra port 3010 có bị chiếm không
lsof -i :3010

# Restart frontend
pm2 restart qa-chatbot-frontend
```

### Backend không chạy
```bash
# Kiểm tra logs
pm2 logs qa-chatbot-backend

# Kiểm tra port 8080 có bị chiếm không
lsof -i :8080

# Restart backend
pm2 restart qa-chatbot-backend
```

### Cargo build failed
```bash
# Xóa cache và build lại
cd rust-backend
rm -rf target
cargo clean
cargo build
cd ..

# Khởi động lại
pm2 restart ecosystem.config.js
```

### Node modules bị lỗi
```bash
# Xóa và cài lại
rm -rf node_modules
npm install

# Khởi động lại
pm2 restart qa-chatbot-frontend
```

## 🧹 Dọn dẹp

### Xóa tất cả processes
```bash
pm2 delete all
```

### Clear logs
```bash
pm2 flush
```

### Reset hoàn toàn
```bash
# Dừng và xóa tất cả
pm2 delete all

# Xóa logs
rm -rf logs/*

# Khởi động lại từ đầu
pm2 start ecosystem.config.js
```

## 📊 Kiểm tra hiệu suất

### Xem CPU và Memory
```bash
pm2 monit
```

### Xem thống kê
```bash
pm2 stats
```

## ✅ Checklist sau khi setup

- [ ] `./setup.sh` chạy thành công
- [ ] `pm2 start ecosystem.config.js` không có lỗi
- [ ] `pm2 status` hiển thị 2 processes "online"
- [ ] Truy cập http://localhost:3010 thấy giao diện
- [ ] Kiểm tra WebSocket connection tại ws://localhost:8080/ws

## 🎯 Tiếp theo

1. Mở trình duyệt: http://localhost:3010
2. Tạo ticket mới
3. Kéo ticket vào cột "Đang Xử Lý" để trigger phân tích
4. Xem kết quả phân tích real-time

## 📚 Tài liệu tham khảo

- [README.md](README.md) - Tài liệu chi tiết
- [PM2 Documentation](https://pm2.keymetrics.io/docs/usage/quick-start/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Next.js Documentation](https://nextjs.org/docs)


