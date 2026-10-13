import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from 'react'
import type { ReactNode } from 'react'

import { api } from '@/lib/api'
import { useAuth } from '@/contexts/auth'
import type { CreateProfileInput, Profile, UpdateProfileInput } from '@/lib/types'

interface ProfilesContextValue {
  profiles: Profile[]
  activeProfile: Profile | null
  loading: boolean
  activeId: string | null
  setActive: (id: string) => void
  refresh: () => Promise<Profile[]>
  createProfile: (data: CreateProfileInput) => Promise<Profile>
  updateProfile: (id: string, data: UpdateProfileInput) => Promise<Profile>
  deleteProfile: (id: string) => Promise<void>
  setDefault: (id: string) => Promise<void>
  uploadCv: (id: string, file: File) => Promise<Profile>
  analyze: (id: string) => Promise<Profile>
}

const ProfilesContext = createContext<ProfilesContextValue | null>(null)

function storageKey(userId: string) {
  return `workeo_active_profile:${userId}`
}

export function ProfilesProvider({ children }: { children: ReactNode }) {
  const { user } = useAuth()
  const [profiles, setProfiles] = useState<Profile[]>([])
  const [activeId, setActiveIdState] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  const refresh = useCallback(async () => {
    if (!user) {
      setProfiles([])
      return []
    }
    setLoading(true)
    try {
      const list = await api.listProfiles()
      setProfiles(list)
      return list
    } finally {
      setLoading(false)
    }
  }, [user])

  // Charge les profils à la connexion et résout le profil actif.
  useEffect(() => {
    if (!user) {
      setProfiles([])
      setActiveIdState(null)
      return
    }
    let active = true
    ;(async () => {
      const list = await refresh()
      if (!active) return
      const stored = localStorage.getItem(storageKey(user.id))
      const resolved =
        list.find((p) => p.id === stored) ??
        list.find((p) => p.is_default) ??
        list[0] ??
        null
      setActiveIdState(resolved?.id ?? null)
    })()
    return () => {
      active = false
    }
  }, [user, refresh])

  const setActive = useCallback(
    (id: string) => {
      setActiveIdState(id)
      if (user) localStorage.setItem(storageKey(user.id), id)
    },
    [user],
  )

  const createProfile = useCallback(
    async (data: CreateProfileInput) => {
      const profile = await api.createProfile(data)
      setProfiles((prev) => [...prev, profile])
      setActive(profile.id)
      return profile
    },
    [setActive],
  )

  const updateProfile = useCallback(
    async (id: string, data: UpdateProfileInput) => {
      const updated = await api.updateProfile(id, data)
      setProfiles((prev) => prev.map((p) => (p.id === id ? updated : p)))
      return updated
    },
    [],
  )

  const deleteProfile = useCallback(
    async (id: string) => {
      await api.deleteProfile(id)
      const next = await refresh()
      setActiveIdState((current) => {
        if (current && current !== id) return current
        const fallback = next.find((p) => p.is_default) ?? next[0] ?? null
        return fallback?.id ?? null
      })
    },
    [refresh],
  )

  const setDefault = useCallback(
    async (id: string) => {
      await api.setDefaultProfile(id)
      const next = await refresh()
      setActive(id)
      void next
    },
    [refresh, setActive],
  )

  const uploadCv = useCallback(async (id: string, file: File) => {
    const updated = await api.uploadCv(id, file)
    setProfiles((prev) => prev.map((p) => (p.id === id ? updated : p)))
    return updated
  }, [])

  const analyze = useCallback(async (id: string) => {
    const updated = await api.analyzeProfile(id)
    setProfiles((prev) => prev.map((p) => (p.id === id ? updated : p)))
    return updated
  }, [])

  const activeProfile =
    profiles.find((p) => p.id === activeId) ?? profiles[0] ?? null

  return (
    <ProfilesContext.Provider
      value={{
        profiles,
        activeProfile,
        activeId: activeProfile?.id ?? null,
        loading,
        setActive,
        refresh,
        createProfile,
        updateProfile,
        deleteProfile,
        setDefault,
        uploadCv,
        analyze,
      }}
    >
      {children}
    </ProfilesContext.Provider>
  )
}

// eslint-disable-next-line react-refresh/only-export-components
export function useProfiles() {
  const ctx = useContext(ProfilesContext)
  if (!ctx)
    throw new Error('useProfiles doit être utilisé dans <ProfilesProvider>')
  return ctx
}
