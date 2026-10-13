// Types alignés sur les modèles Rust (src/models.rs).

export interface User {
  id: string
  email: string
  full_name: string
  avatar_url: string | null
  created_at: string
  updated_at: string
}

export interface AuthResponse {
  user: User
  token: string
}

export interface Language {
  name: string
  level: string | null
}

export interface Education {
  degree: string | null
  school: string | null
  year: string | null
}

export interface Experience {
  title: string | null
  company: string | null
  start: string | null
  end: string | null
  description: string | null
}

export interface Profile {
  id: string
  user_id: string
  name: string
  full_name: string
  email: string | null
  phone: string | null
  location: string | null
  headline: string | null
  summary: string | null
  seniority: string | null
  years_experience: number | null
  skills: string[]
  languages: Language[]
  education: Education[]
  experiences: Experience[]
  domains: string[]
  target_roles: string[]
  search_keywords: string[]
  category: string | null
  remote_preference: string | null
  contract_types: string[]
  salary_min: number | null
  cv_filename: string | null
  cv_text: string | null
  is_default: boolean
  created_at: string
  updated_at: string
}

export interface Job {
  id: string
  source: string
  external_id: string
  title: string
  company: string
  location: string | null
  remote: boolean
  description: string | null
  url: string | null
  summary: string | null
  salary_min: number | null
  salary_max: number | null
  salary_currency: string | null
  contract_type: string | null
  seniority: string | null
  category: string | null
  skills: string[]
  posted_at: string | null
  scraped_at: string
}

export interface ScoredJob extends Job {
  score: number
  reasons: string[]
}

export interface Application {
  id: string
  profile_id: string
  job_id: string
  status: ApplicationStatus
  notes: string | null
  created_at: string
  updated_at: string
}

export interface ApplicationWithJob extends Application {
  job: Job | null
}

export type ApplicationStatus =
  | 'saved'
  | 'applied'
  | 'interview'
  | 'offer'
  | 'rejected'

export interface SearchRun {
  id: string
  profile_id: string
  query: string | null
  location: string | null
  sources: string[]
  status: string
  jobs_found: number
  jobs_new: number
  started_at: string
  finished_at: string | null
}

export interface SearchResult {
  run: SearchRun
  total: number
  results: ScoredJob[]
  source_errors: string[]
}

export interface CreateProfileInput {
  name?: string
  full_name: string
  email?: string
  phone?: string
  location?: string
  headline?: string
  remote_preference?: string
  contract_types?: string[]
  salary_min?: number
  cv_text?: string
  is_default?: boolean
}

export interface UpdateProfileInput {
  name?: string
  full_name?: string
  email?: string
  phone?: string
  location?: string
  headline?: string
  summary?: string
  remote_preference?: string
  contract_types?: string[]
  salary_min?: number
  skills?: string[]
  target_roles?: string[]
  is_default?: boolean
}

export interface SearchInput {
  profile_id: string
  query?: string
  location?: string
  remote_only?: boolean
  contract_type?: string
  latitude?: number
  longitude?: number
  sources?: string[]
}

export interface HealthResponse {
  status: string
  version: string
  llm: {
    provider: string
    model: string
    enabled: boolean
    reachable: boolean
    mode: string
  }
  sources: { name: string; label: string }[]
}

export interface SourcesResponse {
  active_default: string[]
  sources: { name: string; label: string; enabled: boolean }[]
}

export interface StatsResponse {
  jobs: number
  profiles: number
  remote_jobs: number
  matches: number
  applications: number
  searches: number
  by_status: { status: string; count: number }[]
  by_category: { category: string | null; count: number }[]
}
