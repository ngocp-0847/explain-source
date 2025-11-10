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

  getLogs: async (id: string, options?: { limit?: number; offset?: number }) => {
    const params = new URLSearchParams();
    if (options?.limit !== undefined) {
      params.append('limit', options.limit.toString());
    }
    if (options?.offset !== undefined) {
      params.append('offset', options.offset.toString());
    }
    const queryString = params.toString();
    const url = `${API_BASE}/tickets/${id}/logs${queryString ? `?${queryString}` : ''}`;
    const res = await fetch(url);
    if (!res.ok) throw new Error('Failed to get ticket logs');
    return res.json();
  },

  stopAnalysis: async (ticketId: string) => {
    const res = await fetch(`${API_BASE}/tickets/${ticketId}/stop-analysis`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    });
    if (!res.ok) throw new Error('Failed to stop analysis');
    return res.json();
  },
};

