import type {
  ApplicationWithJob,
  AuthResponse,
  CreateProfileInput,
  HealthResponse,
  Job,
  Profile,
  ScoredJob,
  SearchInput,
  SearchResult,
  SearchRun,
  SourcesResponse,
  StatsResponse,
  UpdateProfileInput,
  User,
} from './types'

const TOKEN_KEY = 'workeo_token'

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}
export function setToken(token: string | null) {
  if (token) localStorage.setItem(TOKEN_KEY, token)
  else localStorage.removeItem(TOKEN_KEY)
}

/** Émis lorsqu'une requête renvoie 401 : l'application déconnecte l'utilisateur. */
export const UNAUTHORIZED_EVENT = 'workeo:unauthorized'

export class ApiError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.status = status
    this.name = 'ApiError'
  }
}

async function request<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const headers = new Headers(options.headers)
  const token = getToken()
  if (token) headers.set('Authorization', `Bearer ${token}`)
  if (options.body && !(options.body instanceof FormData)) {
    headers.set('Content-Type', 'application/json')
  }

  const res = await fetch(path, { ...options, headers })

  if (res.status === 401) {
    window.dispatchEvent(new Event(UNAUTHORIZED_EVENT))
    throw new ApiError(401, 'Session expirée, veuillez vous reconnecter.')
  }

  if (!res.ok) {
    let message = `Erreur ${res.status}`
    try {
      const data = await res.json()
      message = data?.error ?? data?.message ?? message
    } catch {
      /* ignore */
    }
    throw new ApiError(res.status, message)
  }

  if (res.status === 204) return undefined as T
  const text = await res.text()
  if (!text) return undefined as T
  return JSON.parse(text) as T
}

const json = (body: unknown): RequestInit => ({ body: JSON.stringify(body) })

export const api = {
  // ─── Auth ───
  register: (data: { full_name: string; email: string; password: string }) =>
    request<AuthResponse>('/api/auth/register', {
      method: 'POST',
      ...json(data),
    }),

  login: (data: { email: string; password: string }) =>
    request<AuthResponse>('/api/auth/login', {
      method: 'POST',
      ...json(data),
    }),

  me: () => request<User>('/api/auth/me'),

  // ─── Méta ───
  health: () => request<HealthResponse>('/api/health'),
  sources: () => request<SourcesResponse>('/api/sources'),
  stats: (profileId?: string) =>
    request<StatsResponse>(
      `/api/stats${profileId ? `?profile_id=${encodeURIComponent(profileId)}` : ''}`,
    ),

  // ─── Profils ───
  listProfiles: () => request<Profile[]>('/api/profiles'),
  createProfile: (data: CreateProfileInput) =>
    request<Profile>('/api/profiles', { method: 'POST', ...json(data) }),
  getProfile: (id: string) => request<Profile>(`/api/profiles/${id}`),
  updateProfile: (id: string, data: UpdateProfileInput) =>
    request<Profile>(`/api/profiles/${id}`, { method: 'PUT', ...json(data) }),
  deleteProfile: (id: string) =>
    request<{ ok: boolean }>(`/api/profiles/${id}`, { method: 'DELETE' }),
  setDefaultProfile: (id: string) =>
    request<Profile>(`/api/profiles/${id}/default`, { method: 'POST' }),

  uploadCv: (id: string, file: File) => {
    const form = new FormData()
    form.append('file', file)
    return request<Profile>(`/api/profiles/${id}/cv`, {
      method: 'POST',
      body: form,
    })
  },

  analyzeProfile: (id: string) =>
    request<Profile>(`/api/profiles/${id}/analyze`, { method: 'POST' }),

  recommendations: (id: string) =>
    request<ScoredJob[]>(`/api/profiles/${id}/recommendations`),

  searches: (id: string) => request<SearchRun[]>(`/api/profiles/${id}/searches`),

  // ─── Recherche ───
  search: (input: SearchInput) =>
    request<SearchResult>('/api/search', { method: 'POST', ...json(input) }),

  // ─── Offres ───
  listJobs: (params: {
    q?: string
    remote?: boolean
    contract?: string
    category?: string
    limit?: number
    offset?: number
  } = {}) => {
    const query = new URLSearchParams()
    if (params.q) query.set('q', params.q)
    if (params.remote) query.set('remote', 'true')
    if (params.contract) query.set('contract', params.contract)
    if (params.category) query.set('category', params.category)
    if (params.limit) query.set('limit', String(params.limit))
    if (params.offset) query.set('offset', String(params.offset))
    const qs = query.toString()
    return request<Job[]>(`/api/jobs${qs ? `?${qs}` : ''}`)
  },
  getJob: (id: string, profileId?: string) =>
    request<ScoredJob>(
      `/api/jobs/${id}${profileId ? `?profile_id=${encodeURIComponent(profileId)}` : ''}`,
    ),

  // ─── Candidatures ───
  createApplication: (data: {
    profile_id: string
    job_id: string
    status?: string
    notes?: string
  }) =>
    request('/api/applications', { method: 'POST', ...json(data) }),
  updateApplication: (
    id: string,
    data: { status?: string; notes?: string },
  ) =>
    request(`/api/applications/${id}`, { method: 'PATCH', ...json(data) }),
  deleteApplication: (id: string) =>
    request(`/api/applications/${id}`, { method: 'DELETE' }),
  listApplications: (profileId: string) =>
    request<ApplicationWithJob[]>(`/api/profiles/${profileId}/applications`),
}
