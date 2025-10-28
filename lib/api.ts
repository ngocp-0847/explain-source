const API_BASE = 'http://localhost:9000/api';

export interface CreateProjectData {
  name: string;
  description?: string;
  directory_path: string;
}

export interface UpdateProjectData {
  name: string;
  description?: string;
  directory_path: string;
}

export interface CreateTicketData {
  title: string;
  description: string;
  status: string;
  code_context?: string;
}

export interface UpdateStatusData {
  status: string;
}

export const projectApi = {
  list: async () => {
    const res = await fetch(`${API_BASE}/projects`);
    if (!res.ok) throw new Error('Failed to list projects');
    return res.json();
  },

  get: async (id: string) => {
    const res = await fetch(`${API_BASE}/projects/${id}`);
    if (!res.ok) throw new Error('Failed to get project');
    return res.json();
  },

  create: async (data: CreateProjectData) => {
    const res = await fetch(`${API_BASE}/projects`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!res.ok) throw new Error('Failed to create project');
    return res.json();
  },

  update: async (id: string, data: UpdateProjectData) => {
    const res = await fetch(`${API_BASE}/projects/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!res.ok) throw new Error('Failed to update project');
    return res.json();
  },

  delete: async (id: string) => {
    const res = await fetch(`${API_BASE}/projects/${id}`, {
      method: 'DELETE',
    });
    if (!res.ok) throw new Error('Failed to delete project');
    return res.status === 204;
  },
};

export const ticketApi = {
  list: async (projectId: string) => {
    const res = await fetch(`${API_BASE}/projects/${projectId}/tickets`);
    if (!res.ok) throw new Error('Failed to list tickets');
    return res.json();
  },

  create: async (projectId: string, data: CreateTicketData) => {
    const res = await fetch(`${API_BASE}/projects/${projectId}/tickets`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!res.ok) throw new Error('Failed to create ticket');
    return res.json();
  },

  updateStatus: async (id: string, status: string) => {
    const res = await fetch(`${API_BASE}/tickets/${id}/status`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status }),
    });
    if (!res.ok) throw new Error('Failed to update ticket status');
    return res.status === 204;
  },

  getLogs: async (id: string) => {
    const res = await fetch(`${API_BASE}/tickets/${id}/logs`);
    if (!res.ok) throw new Error('Failed to get ticket logs');
    return res.json();
  },
};

