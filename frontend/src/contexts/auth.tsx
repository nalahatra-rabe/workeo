import { createContext, useContext, useEffect, useState } from 'react'

import { api, getToken, setToken, UNAUTHORIZED_EVENT } from '@/lib/api'
import type { User } from '@/lib/types'

interface AuthContextValue {
  user: User | null
  loading: boolean
  login: (email: string, password: string) => Promise<void>
  register: (fullName: string, email: string, password: string) => Promise<void>
  logout: () => void
  refresh: () => Promise<void>
}

const AuthContext = createContext<AuthContextValue | null>(null)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState(true)

  const loadUser = async () => {
    if (!getToken()) {
      setUser(null)
      return
    }
    try {
      const me = await api.me()
      setUser(me)
    } catch {
      setToken(null)
      setUser(null)
    }
  }

  useEffect(() => {
    let active = true
    ;(async () => {
      await loadUser()
      if (active) setLoading(false)
    })()
    const onUnauthorized = () => setUser(null)
    window.addEventListener(UNAUTHORIZED_EVENT, onUnauthorized)
    return () => {
      active = false
      window.removeEventListener(UNAUTHORIZED_EVENT, onUnauthorized)
    }
  }, [])

  const login = async (email: string, password: string) => {
    const res = await api.login({ email, password })
    setToken(res.token)
    setUser(res.user)
  }

  const register = async (
    full_name: string,
    email: string,
    password: string,
  ) => {
    const res = await api.register({ full_name, email, password })
    setToken(res.token)
    setUser(res.user)
  }

  const logout = () => {
    setToken(null)
    setUser(null)
  }

  return (
    <AuthContext.Provider
      value={{ user, loading, login, register, logout, refresh: loadUser }}
    >
      {children}
    </AuthContext.Provider>
  )
}

// eslint-disable-next-line react-refresh/only-export-components
export function useAuth() {
  const ctx = useAuthContext()
  return ctx
}

function useAuthContext() {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth doit être utilisé dans <AuthProvider>')
  return ctx
}
