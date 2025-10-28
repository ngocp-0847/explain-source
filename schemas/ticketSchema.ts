import { z } from 'zod'

export const ticketFormSchema = z.object({
  title: z
    .string()
    .min(1, 'Tiêu đề không được trống')
    .max(100, 'Tiêu đề không được quá 100 ký tự'),
  description: z
    .string()
    .min(10, 'Mô tả phải có ít nhất 10 ký tự')
    .max(500, 'Mô tả không được quá 500 ký tự'),
  codeContext: z.string().optional(),
})

export type TicketFormValues = z.infer<typeof ticketFormSchema>

