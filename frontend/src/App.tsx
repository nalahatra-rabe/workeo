import { Navigate, Route, Routes } from 'react-router-dom'

import { AppLayout } from './components/layout/app-layout'
import { FullScreenLoader } from './components/full-screen-loader'
import { useAuth } from './contexts/auth'
import { useProfiles } from './contexts/profiles'
import { ApplicationsPage } from './pages/applications'
import { DashboardPage } from './pages/dashboard'
import { JobsPage } from './pages/jobs'
import { LoginPage } from './pages/login'
import { OnboardingPage } from './pages/onboarding'
import { ProfilePage } from './pages/profile'
import { ProfileListPage } from './pages/profiles'
import { RegisterPage } from './pages/register'
import { SearchPage } from './pages/search'

function RequireAuth({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuth()
  if (loading) return <FullScreenLoader />
  if (!user) return <Navigate to="/login" replace />
  return <>{children}</>
}

function RequireProfiles({ children }: { children: React.ReactNode }) {
  const { profiles, loading } = useProfiles()
  if (loading && profiles.length === 0) {
    return <FullScreenLoader label="Chargement des profils…" />
  }
  if (profiles.length === 0) return <Navigate to="/onboarding" replace />
  return <>{children}</>
}

function PublicOnly({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuth()
  if (loading) return <FullScreenLoader />
  if (user) return <Navigate to="/" replace />
  return <>{children}</>
}

export default function App() {
  return (
    <Routes>
      <Route
        path="/login"
        element={
          <PublicOnly>
            <LoginPage />
          </PublicOnly>
        }
      />
      <Route
        path="/register"
        element={
          <PublicOnly>
            <RegisterPage />
          </PublicOnly>
        }
      />
      <Route
        path="/onboarding"
        element={
          <RequireAuth>
            <OnboardingPage />
          </RequireAuth>
        }
      />

      <Route
        element={
          <RequireAuth>
            <RequireProfiles>
              <AppLayout />
            </RequireProfiles>
          </RequireAuth>
        }
      >
        <Route path="/" element={<DashboardPage />} />
        <Route path="/search" element={<SearchPage />} />
        <Route path="/jobs" element={<JobsPage />} />
        <Route path="/applications" element={<ApplicationsPage />} />
        <Route path="/profiles" element={<ProfileListPage />} />
        <Route path="/profile" element={<ProfilePage />} />
      </Route>

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  )
}
