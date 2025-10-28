'use client'

import { useEffect } from 'react'
import { useProjectStore } from '@/stores/projectStore'
import { useWebSocketStore } from '@/stores/websocketStore'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Folder, Trash2, Settings2 } from 'lucide-react'
import { useRouter } from 'next/navigation'

interface ProjectListProps {
  onCreateProject: () => void
}

export function ProjectList({ onCreateProject }: ProjectListProps) {
  const { projects, setProjects } = useProjectStore()
  const { isConnected, send } = useWebSocketStore()
  const router = useRouter()

  useEffect(() => {
    if (isConnected) {
      send({ type: 'load-projects' })
    }
  }, [isConnected, send])

  const handleProjectClick = (projectId: string) => {
    router.push(`/projects/${projectId}`)
  }

  const handleDelete = (projectId: string, e: React.MouseEvent) => {
    e.stopPropagation()
    if (confirm('Bạn có chắc muốn xóa project này?')) {
      send({ type: 'delete-project', projectId })
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">Projects</h2>
          <p className="text-gray-600 mt-1">Quản lý các dự án của bạn</p>
        </div>
        <Button onClick={onCreateProject}>Tạo Project Mới</Button>
      </div>

      {projects.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Folder className="w-12 h-12 text-gray-400 mb-4" />
            <h3 className="text-lg font-semibold text-gray-900 mb-2">
              Chưa có project nào
            </h3>
            <p className="text-gray-600 mb-4 text-center max-w-sm">
              Tạo project đầu tiên để bắt đầu phân tích code và quản lý tickets
            </p>
            <Button onClick={onCreateProject}>Tạo Project Mới</Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {projects.map((project) => (
            <Card
              key={project.id}
              className="cursor-pointer hover:shadow-lg transition-shadow"
              onClick={() => handleProjectClick(project.id)}
            >
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <CardTitle className="text-lg">{project.name}</CardTitle>
                    <CardDescription className="mt-1">
                      {project.description || 'Không có mô tả'}
                    </CardDescription>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  <div>
                    <p className="text-sm text-gray-600 mb-1">Directory Path</p>
                    <p className="text-sm font-mono bg-gray-50 px-2 py-1 rounded truncate">
                      {project.directoryPath}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={(e) => {
                        e.stopPropagation()
                        // Handle edit
                      }}
                    >
                      <Settings2 className="w-4 h-4" />
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={(e) => handleDelete(project.id, e)}
                      className="text-red-600 hover:text-red-700 hover:border-red-300"
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}

