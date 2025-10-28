'use client'

import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { useProjectStore } from '@/stores/projectStore'
import { useWebSocketStore } from '@/stores/websocketStore'
import { Project } from '@/types/ticket'

interface ProjectFormDialogProps {
  isOpen: boolean
  onClose: () => void
  project?: Project | null
}

interface ProjectFormValues {
  name: string
  description: string
  directoryPath: string
}

export function ProjectFormDialog({ isOpen, onClose, project }: ProjectFormDialogProps) {
  const { addProject, updateProject } = useProjectStore()
  const { send } = useWebSocketStore()
  const [isSubmitting, setIsSubmitting] = useState(false)

  const form = useForm<ProjectFormValues>({
    defaultValues: {
      name: '',
      description: '',
      directoryPath: '',
    },
  })

  useEffect(() => {
    if (project) {
      form.reset({
        name: project.name,
        description: project.description || '',
        directoryPath: project.directoryPath,
      })
    } else {
      form.reset({
        name: '',
        description: '',
        directoryPath: '',
      })
    }
  }, [project, form])

  const onSubmit = async (data: ProjectFormValues) => {
    setIsSubmitting(true)

    try {
      if (project) {
        // Edit mode
        const updatedProject: Partial<Project> = {
          name: data.name,
          description: data.description || undefined,
          directoryPath: data.directoryPath,
        }

        updateProject(project.id, updatedProject)

        send({
          type: 'update-project',
          id: project.id,
          name: data.name,
          description: data.description || undefined,
          directoryPath: data.directoryPath,
        })
      } else {
        // Create mode
        const projectId = crypto.randomUUID()
        const now = new Date().toISOString()

        const newProject: Project = {
          id: projectId,
          name: data.name,
          description: data.description || undefined,
          directoryPath: data.directoryPath,
          createdAt: now,
          updatedAt: now,
        }

        addProject(newProject)

        send({
          type: 'create-project',
          id: projectId,
          name: data.name,
          description: data.description || undefined,
          directoryPath: data.directoryPath,
        })
      }

      onClose()
      form.reset()
    } catch (error) {
      console.error('Error submitting project:', error)
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{project ? 'Chỉnh sửa Project' : 'Tạo Project Mới'}</DialogTitle>
        </DialogHeader>

        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="name">Tên Project *</Label>
            <Input
              id="name"
              placeholder="Ví dụ: E-commerce Platform"
              {...form.register('name', { required: 'Tên project là bắt buộc' })}
            />
            {form.formState.errors.name && (
              <p className="text-sm text-red-500">{form.formState.errors.name.message}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">Mô tả</Label>
            <Textarea
              id="description"
              placeholder="Mô tả về project..."
              rows={3}
              {...form.register('description')}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="directoryPath">Directory Path *</Label>
            <Input
              id="directoryPath"
              placeholder="/path/to/project"
              {...form.register('directoryPath', { required: 'Directory path là bắt buộc' })}
            />
            <p className="text-xs text-gray-500">
              Đường dẫn local đến thư mục source code của project trên server
            </p>
            {form.formState.errors.directoryPath && (
              <p className="text-sm text-red-500">{form.formState.errors.directoryPath.message}</p>
            )}
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose} disabled={isSubmitting}>
              Hủy
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {project ? 'Cập nhật' : 'Tạo Project'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

