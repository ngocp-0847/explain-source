'use client'

import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { useTicketStore } from '@/stores/ticketStore'
import { useUIStore } from '@/stores/uiStore'
import { useProjectStore } from '@/stores/projectStore'
import { useWebSocketStore } from '@/stores/websocketStore'
import { ticketFormSchema, TicketFormValues } from '@/schemas/ticketSchema'
import { Ticket, TicketStatus } from '@/types/ticket'
import { ticketApi } from '@/lib/api'

export function TicketFormDialog() {
  const { isOpen, onClose, selectedTicket, setSelectedTicket } = useUIStore(state => ({
    isOpen: state.isTicketModalOpen,
    onClose: state.closeTicketModal,
    selectedTicket: state.selectedTicketForEdit,
    setSelectedTicket: state.setSelectedTicketForEdit,
  }))
  
  const addTicket = useTicketStore(state => state.addTicket)
  const updateTicket = useTicketStore(state => state.updateTicket)
  const selectedProjectId = useProjectStore(state => state.selectedProjectId)
  const { send } = useWebSocketStore()

  const form = useForm<TicketFormValues>({
    resolver: zodResolver(ticketFormSchema),
    defaultValues: {
      title: '',
      description: '',
      codeContext: '',
    },
  })

  useEffect(() => {
    if (selectedTicket) {
      form.reset({
        title: selectedTicket.title,
        description: selectedTicket.description,
        codeContext: selectedTicket.codeContext || '',
      })
    } else {
      form.reset({
        title: '',
        description: '',
        codeContext: '',
      })
    }
  }, [selectedTicket, form])

  const onSubmit = async (data: TicketFormValues) => {
    if (selectedTicket) {
      // Edit mode - update existing ticket
      updateTicket(selectedTicket.id, {
        title: data.title,
        description: data.description,
        codeContext: data.codeContext || undefined,
      })
      
      // Sync to backend
      send({
        type: 'update-ticket',
        id: selectedTicket.id,
        title: data.title,
        description: data.description,
        codeContext: data.codeContext || undefined,
      })
    } else {
      // Create mode - add new ticket via REST API
      if (!selectedProjectId) {
        alert('Vui lòng chọn project trước khi tạo ticket')
        return
      }
      
      try {
        // Call REST API to create ticket
        const createdTicket = await ticketApi.create(selectedProjectId, {
          title: data.title,
          description: data.description,
          status: 'todo',
          code_context: data.codeContext || undefined,
        })
        
        // Add to local store with proper mapping
        addTicket({
          id: createdTicket.id,
          projectId: createdTicket.project_id,
          title: createdTicket.title,
          description: createdTicket.description,
          status: createdTicket.status as TicketStatus,
          createdAt: new Date(createdTicket.created_at),
          updatedAt: createdTicket.updated_at ? new Date(createdTicket.updated_at) : undefined,
          codeContext: createdTicket.code_context || undefined,
          analysisResult: createdTicket.analysis_result || undefined,
          isAnalyzing: createdTicket.is_analyzing,
          logs: [], // Initialize empty logs array
        })
        
        console.log('✅ Ticket created successfully via REST API:', createdTicket.id)
      } catch (error) {
        console.error('❌ Failed to create ticket:', error)
        alert('Lỗi khi tạo ticket. Vui lòng thử lại.')
        return
      }
    }

    onClose()
    setSelectedTicket(null)
    form.reset()
  }

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{selectedTicket ? 'Chỉnh sửa Ticket' : 'Tạo Ticket Mới'}</DialogTitle>
        </DialogHeader>

        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="title">Tiêu đề *</Label>
            <Input
              id="title"
              placeholder="Ví dụ: Hiểu flow đăng nhập user"
              {...form.register('title')}
            />
            {form.formState.errors.title && (
              <p className="text-sm text-red-500">{form.formState.errors.title.message}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">Mô tả câu hỏi *</Label>
            <Textarea
              id="description"
              placeholder="Mô tả chi tiết câu hỏi về business flow..."
              rows={3}
              {...form.register('description')}
            />
            {form.formState.errors.description && (
              <p className="text-sm text-red-500">{form.formState.errors.description.message}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="codeContext">Code Context (tùy chọn)</Label>
            <Input
              id="codeContext"
              placeholder="Ví dụ: auth/login.js, api/payment.js"
              {...form.register('codeContext')}
            />
            <p className="text-xs text-gray-500">
              Đường dẫn file hoặc module code liên quan
            </p>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              Hủy
            </Button>
            <Button type="submit">
              {selectedTicket ? 'Cập nhật' : 'Tạo Ticket'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
