import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export interface User {
  id: string
  username: string
}

interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  isLoading: boolean
  hasHydrated: boolean
  login: (username: string, password: string) => Promise<void>
  register: (username: string, password: string) => Promise<void>
  logout: () => void
  setAuth: (user: User, token: string) => void
  verifyAuth: () => Promise<void>
  init: () => Promise<void>
  setHasHydrated: (hasHydrated: boolean) => void
}

const API_BASE = 'http://localhost:9000/api'

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: true,
      hasHydrated: false,

      login: async (username: string, password: string) => {
        const response = await fetch(`${API_BASE}/auth/login`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ username, password }),
        })

        if (!response.ok) {
          const error = await response.json()
          throw new Error(error.error || 'Login failed')
        }

        const data = await response.json()
        set({
          user: data.user,
          token: data.token,
          isAuthenticated: true,
          isLoading: false,
        })
      },

      register: async (username: string, password: string) => {
        const response = await fetch(`${API_BASE}/auth/register`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ username, password }),
        })

        if (!response.ok) {
          const error = await response.json()
          throw new Error(error.error || 'Registration failed')
        }

        const data = await response.json()
        set({
          user: data.user,
          token: data.token,
          isAuthenticated: true,
          isLoading: false,
        })
      },

      logout: () => {
        set({
          user: null,
          token: null,
          isAuthenticated: false,
          isLoading: false,
        })
      },

      setAuth: (user: User, token: string) => {
        set({
          user,
          token,
          isAuthenticated: true,
          isLoading: false,
        })
      },

      verifyAuth: async () => {
        const state = get()
        const token = state.token

        if (!token) {
          set({
            isAuthenticated: false,
            isLoading: false,
            user: null,
          })
          return
        }

        try {
          const response = await fetch(`${API_BASE}/auth/me`, {
            headers: {
              'Authorization': `Bearer ${token}`,
            },
          })

          if (!response.ok) {
            // Token invalid or expired
            set({
              user: null,
              token: null,
              isAuthenticated: false,
              isLoading: false,
            })
            return
          }

          const user = await response.json()
          set({
            user,
            isAuthenticated: true,
            isLoading: false,
          })
        } catch (error) {
          console.error('Failed to verify auth:', error)
          set({
            user: null,
            token: null,
            isAuthenticated: false,
            isLoading: false,
          })
        }
      },

      init: async () => {
        // Only verify if we have a token (after hydration)
        const state = get()
        if (!state.hasHydrated) {
          // Wait for hydration
          return
        }
        
        set({ isLoading: true })
        await get().verifyAuth()
      },

      setHasHydrated: (hasHydrated: boolean) => {
        set({ hasHydrated })
        // After hydration, verify auth if we have a token
        const state = get()
        if (hasHydrated && state.token) {
          get().init()
        } else if (hasHydrated && !state.token) {
          // No token after hydration, set loading to false
          set({ isLoading: false, isAuthenticated: false })
        }
      },
    }),
    {
      name: 'auth-storage',
      // Only persist these fields
      partialize: (state) => ({
        token: state.token,
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
      // Callback when rehydration is complete
      onRehydrateStorage: () => (state) => {
        if (state) {
          state.setHasHydrated(true)
        }
      },
    }
  )
)

