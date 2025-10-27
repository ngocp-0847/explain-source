# Multi-stage build cho Next.js + Rust
FROM node:18-alpine AS frontend-builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

# Rust builder stage
FROM rust:1.75-alpine AS rust-builder

WORKDIR /app
COPY rust-backend/ .
RUN cargo build --release

# Final stage
FROM node:18-alpine

WORKDIR /app

# Copy frontend build
COPY --from=frontend-builder /app/.next ./.next
COPY --from=frontend-builder /app/public ./public
COPY --from=frontend-builder /app/package*.json ./
COPY --from=frontend-builder /app/node_modules ./node_modules
COPY --from=frontend-builder /app/next.config.js ./

# Copy Rust binary
COPY --from=rust-builder /app/target/release/qa-chatbot-backend ./backend

# Install dependencies
RUN apk add --no-cache curl

# Expose ports
EXPOSE 3010 8080

# Start script
COPY start-docker.sh ./
RUN chmod +x start-docker.sh

CMD ["./start-docker.sh"]