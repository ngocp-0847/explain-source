'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { useAuthStore } from '@/stores/authStore'

export default function Home() {
  const router = useRouter()
  const { isAuthenticated, isLoading, hasHydrated } = useAuthStore()

  useEffect(() => {
    // Wait for hydration to complete before checking auth
    if (!hasHydrated) return
    if (isLoading) return

    // If not authenticated, redirect to login
    if (!isAuthenticated) {
      router.replace('/login')
      return
    }
    
    // If authenticated, redirect to projects page
    router.replace('/projects')
  }, [router, isAuthenticated, isLoading, hasHydrated])

  return (
    <div className="min-h-screen bg-gray-100 flex items-center justify-center">
      <div className="text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-gray-900 mx-auto mb-4"></div>
        <p className="text-gray-600">
          {!hasHydrated ? 'Đang tải...' : isLoading ? 'Đang xác thực...' : 'Đang chuyển hướng...'}
        </p>
      </div>
    </div>
  )
}
