#!/bin/bash

echo "🚀 Khởi động QA Chatbot MVP..."

# Kiểm tra Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js chưa được cài đặt. Vui lòng cài đặt Node.js trước."
    exit 1
fi

# Kiểm tra Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust chưa được cài đặt. Vui lòng cài đặt Rust trước."
    exit 1
fi

# Cài đặt dependencies frontend
echo "📦 Cài đặt dependencies frontend..."
npm install

# Build Rust backend
echo "🔨 Build Rust backend..."
cd rust-backend
cargo build --release
cd ..

# Chạy Rust backend trong background
echo "🦀 Khởi động Rust backend..."
cd rust-backend
cargo run &
RUST_PID=$!
cd ..

# Đợi backend khởi động
echo "⏳ Đợi backend khởi động..."
sleep 3

# Chạy Next.js frontend
echo "⚛️ Khởi động Next.js frontend..."
npm run dev &
NEXT_PID=$!

echo "✅ Ứng dụng đã khởi động!"
echo "🌐 Frontend: http://localhost:3010"
echo "🔧 Backend: http://localhost:8080"
echo ""
echo "Nhấn Ctrl+C để dừng tất cả services..."

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