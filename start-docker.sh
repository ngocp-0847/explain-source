#!/bin/sh

echo "🚀 Khởi động QA Chatbot MVP trong Docker..."

# Chạy Rust backend trong background
echo "🦀 Khởi động Rust backend..."
./backend &
RUST_PID=$!

# Đợi backend khởi động
echo "⏳ Đợi backend khởi động..."
sleep 3

# Chạy Next.js frontend
echo "⚛️ Khởi động Next.js frontend..."
npm start &
NEXT_PID=$!

echo "✅ Ứng dụng đã khởi động!"
echo "🌐 Frontend: http://localhost:3010"
echo "🔧 Backend: http://localhost:8080"

# Hàm cleanup khi thoát
cleanup() {
    echo ""
    echo "🛑 Đang dừng services..."
    kill $RUST_PID 2>/dev/null
    kill $NEXT_PID 2>/dev/null
    echo "✅ Đã dừng tất cả services"
    exit 0
}

# Bắt tín hiệu để cleanup
trap cleanup SIGINT SIGTERM

# Đợi
wait