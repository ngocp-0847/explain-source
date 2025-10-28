import { create } from 'zustand'
import { Project } from '@/types/ticket'

interface ProjectState {
  projects: Project[]
  selectedProjectId: string | null
  setProjects: (projects: Project[]) => void
  addProject: (project: Project) => void
  updateProject: (id: string, project: Partial<Project>) => void
  deleteProject: (id: string) => void
  selectProject: (id: string | null) => void
  getSelectedProject: () => Project | undefined
}

export const useProjectStore = create<ProjectState>((set, get) => ({
  projects: [],
  selectedProjectId: null,

  setProjects: (projects) => set({ projects }),

  addProject: (project) =>
    set((state) => ({
      projects: [...state.projects, project],
    })),

  updateProject: (id, updates) =>
    set((state) => ({
      projects: state.projects.map((p) =>
        p.id === id ? { ...p, ...updates } : p
      ),
    })),

  deleteProject: (id) =>
    set((state) => ({
      projects: state.projects.filter((p) => p.id !== id),
      selectedProjectId:
        state.selectedProjectId === id ? null : state.selectedProjectId,
    })),

  selectProject: (id) => set({ selectedProjectId: id }),

  getSelectedProject: () => {
    const state = get()
    return state.projects.find((p) => p.id === state.selectedProjectId)
  },
}))

