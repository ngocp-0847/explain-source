#!/bin/bash

# QA Chatbot MVP - Stop Script
# Script để dừng ứng dụng

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🛑 QA Chatbot MVP - Stop Script${NC}"
echo ""

# Kiểm tra PM2 đã được cài đặt chưa
if ! command -v pm2 &> /dev/null; then
    echo "❌ PM2 chưa được cài đặt."
    exit 1
fi

# Kiểm tra xem ứng dụng có đang chạy không
if pm2 list | grep -q "qa-chatbot-frontend\|qa-chatbot-backend"; then
    echo "📊 Trạng thái hiện tại:"
    pm2 status
    echo ""
    read -p "Bạn có chắc chắn muốn dừng ứng dụng? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]
    then
        echo "🛑 Đang dừng ứng dụng..."
        pm2 stop ecosystem.config.js
        echo ""
        echo -e "${GREEN}✅ Ứng dụng đã được dừng!${NC}"
        echo ""
        pm2 status
        echo ""
    else
        echo "✅ Giữ nguyên trạng thái hiện tại"
    fi
else
    echo "ℹ️  Ứng dụng không đang chạy"
    echo ""
    pm2 status
fi

echo ""



