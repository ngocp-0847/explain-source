#!/bin/bash

# QA Chatbot MVP - Setup Script
# Script này sẽ cài đặt và thiết lập môi trường development

set -e

echo "🚀 Bắt đầu cài đặt QA Chatbot MVP..."

# Kiểm tra Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js chưa được cài đặt. Vui lòng cài đặt Node.js 18+ trước."
    exit 1
fi

echo "✅ Node.js đã được cài đặt: $(node --version)"

# Kiểm tra npm
if ! command -v npm &> /dev/null; then
    echo "❌ npm chưa được cài đặt."
    exit 1
fi

echo "✅ npm đã được cài đặt: $(npm --version)"

# Kiểm tra Rust
if ! command -v rustc &> /dev/null; then
    echo "⚠️  Rust chưa được cài đặt. Đang cài đặt Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

echo "✅ Rust đã được cài đặt: $(rustc --version)"

# Kiểm tra PM2
if ! command -v pm2 &> /dev/null; then
    echo "⚠️  PM2 chưa được cài đặt. Đang cài đặt PM2..."
    npm install -g pm2
fi

echo "✅ PM2 đã được cài đặt: $(pm2 --version)"

# Tạo thư mục logs nếu chưa tồn tại
if [ ! -d "logs" ]; then
    mkdir -p logs
    echo "✅ Đã tạo thư mục logs/"
fi

# Cài đặt frontend dependencies
echo "📦 Đang cài đặt frontend dependencies..."
npm install

# Cài đặt backend dependencies
echo "📦 Đang cài đặt backend dependencies..."
cd rust-backend
cargo build
cd ..

echo ""
echo "✅ Cài đặt hoàn tất!"
echo ""

# Hỏi người dùng có muốn chạy ứng dụng ngay không
read -p "🚀 Bạn có muốn khởi động ứng dụng ngay bây giờ? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]
then
    echo "🔄 Đang khởi động ứng dụng với PM2..."
    pm2 start ecosystem.config.js
    
    echo ""
    echo "✅ Ứng dụng đã được khởi động!"
    echo ""
    echo "📊 Xem trạng thái:"
    pm2 status
    
    echo ""
    echo "📝 Các lệnh quản lý ứng dụng:"
    echo "   pm2 logs                              # Xem logs"
    echo "   pm2 logs --lines 50                  # Xem 50 dòng logs cuối"
    echo "   pm2 monit                            # Dashboard monitoring"
    echo "   pm2 stop ecosystem.config.js         # Dừng ứng dụng"
    echo "   pm2 restart ecosystem.config.js       # Khởi động lại"
    echo ""
    echo "🌐 Truy cập ứng dụng tại:"
    echo "   Frontend: http://localhost:3010"
    echo "   Backend API: http://localhost:8080"
    echo "   WebSocket: ws://localhost:8080/ws"
    echo ""
else
    echo ""
    echo "📝 Các lệnh để chạy ứng dụng:"
    echo "   pm2 start ecosystem.config.js        # Khởi động cả frontend và backend"
    echo "   pm2 stop ecosystem.config.js          # Dừng ứng dụng"
    echo "   pm2 restart ecosystem.config.js       # Khởi động lại"
    echo "   pm2 logs                              # Xem logs"
    echo "   pm2 status                            # Xem trạng thái"
    echo ""
    echo "🌐 Truy cập ứng dụng tại:"
    echo "   Frontend: http://localhost:3010"
    echo "   Backend API: http://localhost:8080"
    echo "   WebSocket: ws://localhost:8080/ws"
    echo ""
fi


