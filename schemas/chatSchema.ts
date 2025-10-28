import { z } from 'zod'

export const chatMessageSchema = z.object({
  message: z
    .string()
    .min(1, 'Tin nhắn không được trống')
    .max(500, 'Tin nhắn không được quá 500 ký tự'),
})

export type ChatMessageFormValues = z.infer<typeof chatMessageSchema>

