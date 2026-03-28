import { browser } from '$app/environment';

const API_BASE = '/api';

interface ApiKey {
  id: number;
  name: string;
  key: string;
  description: string | null;
  is_enabled: boolean;
  created_at: string;
  updated_at: string;
}

interface CreateApiKeyRequest {
  name: string;
  description?: string;
}

interface UpdateApiKeyRequest {
  name?: string;
  description?: string;
}

function getAuthHeaders(): Record<string, string> {
  if (!browser) return {};
  const token = localStorage.getItem('token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function fetchApi<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...getAuthHeaders(),
      ...options.headers,
    },
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: 'Unknown error' }));
    throw new Error(error.error || `HTTP ${response.status}`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

export const apiKeysApi = {
  list: () => fetchApi<ApiKey[]>('/keys'),

  create: (data: CreateApiKeyRequest) =>
    fetchApi<ApiKey>('/keys', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: number, data: UpdateApiKeyRequest) =>
    fetchApi<ApiKey>(`/keys/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: number) =>
    fetchApi<void>(`/keys/${id}`, {
      method: 'DELETE',
    }),

  toggle: (id: number) =>
    fetchApi<ApiKey>(`/keys/${id}/toggle`, {
      method: 'POST',
    }),
};

export type { ApiKey, CreateApiKeyRequest, UpdateApiKeyRequest };
