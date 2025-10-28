'use client'

import { useEffect, useState } from 'react'
import { useProjectStore } from '@/stores/projectStore'
import { useWebSocketStore } from '@/stores/websocketStore'
import { ProjectList } from '@/components/ProjectList'
import { ProjectFormDialog } from '@/components/ProjectFormDialog'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Home, Plus } from 'lucide-react'

export default function ProjectsPage() {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const { isConnected } = useWebSocketStore()
  const router = useRouter()

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center gap-4">
              <Button variant="ghost" onClick={() => router.push('/')} size="sm">
                <Home className="w-4 h-4 mr-2" />
                Home
              </Button>
              <h1 className="text-2xl font-bold text-gray-900">Quản lý Projects</h1>
            </div>
            <div className="flex items-center gap-2">
              <Button onClick={() => setIsCreateDialogOpen(true)}>
                <Plus className="w-4 h-4 mr-2" />
                Tạo Project
              </Button>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <ProjectList onCreateProject={() => setIsCreateDialogOpen(true)} />
      </main>

      <ProjectFormDialog
        isOpen={isCreateDialogOpen}
        onClose={() => setIsCreateDialogOpen(false)}
      />
    </div>
  )
}

