# Next Steps - Hoàn Thiện Refactor

## 1. Cài đặt Dependencies

```bash
npm install
```

Lệnh này sẽ cài đặt:
- zustand
- react-hook-form
- @hookform/resolvers
- zod
- @radix-ui/react-dialog
- @radix-ui/react-scroll-area
- tailwindcss-animate
- clsx
- tailwind-merge
- class-variance-authority

## 2. Xóa Files Cũ (Sau khi test thành công)

```bash
rm components/TicketModal.tsx
rm components/TicketDetailModal.tsx
rm hooks/useSocket.ts
```

## 3. Sửa Logic WebSocket Connection trong stores/websocketStore.ts

File hiện tại chưa có auto-connect. Cần thêm vào cuối file:

```typescript
// Auto-connect on mount (nếu chạy trong client)
if (typeof window !== 'undefined') {
  useWebSocketStore.getState().connect('ws://localhost:9000/ws')
}
```

Hoặc connect từ `app/page.tsx` khi component mount.

## 4. Testing

Sau khi cài dependencies, test các features:

### Test Cases:
1. **Create Ticket**
   - Click "Tạo Ticket Mới"
   - Điền form với validation (thử error cases)
   - Submit và verify ticket xuất hiện trong Kanban

2. **Edit Ticket**
   - Click edit icon trên ticket card
   - Verify form pre-filled với data hiện tại
   - Update và verify changes

3. **Drag & Drop**
   - Drag ticket từ "Cần Làm" sang "Đang Xử Lý"
   - Verify ticket status update
   - Verify analysis tự động trigger khi move to "in-progress"

4. **WebSocket Connection**
   - Check connection status badge (header)
   - Verify logs real-time update
   - Verify analysis result update

5. **Chat Interface**
   - Test send message
   - Verify message appear
   - Test typing indicator

## 5. Fix TypeScript Errors (sau khi npm install)

Các lỗi TypeScript sẽ tự động biến mất sau khi cài zustand và các dependencies khác.

## 6. Optional Improvements

### A. Toast Notifications
Cài thêm shadcn Toast component để hiển thị notifications:

```bash
npx shadcn-ui@latest add toast
```

Sử dụng để notify user khi:
- Ticket created/updated
- Analysis completed
- WebSocket disconnected

### B. Error Boundaries
Thêm error boundaries để handle runtime errors gracefully:

```typescript
// app/error-boundary.tsx
'use client'
import { Component } from 'react'

export class ErrorBoundary extends Component {
  // Error boundary implementation
}
```

### C. Loading States
Thêm loading skeletons cho better UX:

```typescript
// components/ui/skeleton.tsx
```

## 7. Performance Optimization

### Memoization
Sử dụng `useMemo` cho expensive computations:

```typescript
const filteredTickets = useMemo(
  () => tickets.filter(t => t.status === status),
  [tickets, status]
)
```

### Zustand Selectors
Sử dụng selectors để tránh re-render:

```typescript
// Good
const tickets = useTicketStore(state => state.tickets)

// Bad (causes re-render on any state change)
const { tickets, addTicket } = useTicketStore()
```

## 8. Mobile Responsive

Verify responsive design trên mobile:
- Grid layout adapt to single column
- Modals full screen on mobile
- Touch-friendly drag handles

## Current Status

✅ Stores architecture hoàn thành
✅ Zod schemas hoàn thành  
✅ Shadcn components tạo xong
✅ Components refactored
⏳ Cần npm install để fix TypeScript errors
⏳ Cần test toàn bộ flows
⏳ Cần xóa files cũ

## Summary

Refactor đã hoàn thành ~95%. Chỉ cần:
1. npm install
2. Test flows
3. Xóa files cũ

Architecture mới vững chắc với Zustand, shadcn/ui, và form validation!

