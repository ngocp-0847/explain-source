#!/bin/bash

# QA Chatbot MVP - Run Script
# Script để chạy ứng dụng với PM2

set -e

# Màu sắc cho output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 QA Chatbot MVP - Run Script${NC}"
echo ""

# Kiểm tra PM2 đã được cài đặt chưa
if ! command -v pm2 &> /dev/null; then
    echo "❌ PM2 chưa được cài đặt."
    echo "Vui lòng chạy: ./setup.sh để cài đặt PM2"
    exit 1
fi

# Kiểm tra xem ứng dụng đã chạy chưa
if pm2 list | grep -q "qa-chatbot-frontend\|qa-chatbot-backend"; then
    echo "⚠️  Ứng dụng đã đang chạy!"
    echo ""
    pm2 status
    echo ""
    read -p "Bạn có muốn restart? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]
    then
        echo "🔄 Đang restart ứng dụng..."
        pm2 restart ecosystem.config.js
    else
        echo "✅ Giữ nguyên trạng thái hiện tại"
        exit 0
    fi
else
    echo "🔄 Đang khởi động ứng dụng với PM2..."
    pm2 start ecosystem.config.js
fi

echo ""
echo -e "${GREEN}✅ Ứng dụng đã được khởi động!${NC}"
echo ""
echo "📊 Trạng thái:"
pm2 status

echo ""
echo -e "${YELLOW}📝 Các lệnh quản lý:${NC}"
echo "   pm2 logs                              # Xem logs"
echo "   pm2 logs --lines 50                  # Xem 50 dòng logs cuối"
echo "   pm2 logs qa-chatbot-frontend         # Xem logs frontend"
echo "   pm2 logs qa-chatbot-backend          # Xem logs backend"
echo "   pm2 monit                            # Dashboard monitoring"
echo "   pm2 stop ecosystem.config.js          # Dừng ứng dụng"
echo "   pm2 restart ecosystem.config.js       # Khởi động lại"
echo ""
echo -e "${YELLOW}🌐 Truy cập ứng dụng:${NC}"
echo "   Frontend: http://localhost:3010"
echo "   Backend API: http://localhost:8080"
echo "   WebSocket: ws://localhost:8080/ws"
echo ""

