import { AnimatePresence, motion } from 'framer-motion'
import {
  Briefcase,
  LayoutDashboard,
  Menu,
  Search,
  Sparkles,
  Users,
  X,
} from 'lucide-react'
import { useState } from 'react'
import { NavLink, Outlet, useLocation } from 'react-router-dom'

import { Logo } from '@/components/logo'
import { AccountMenu } from '@/components/layout/account-menu'
import { ProfileSwitcher } from '@/components/layout/profile-switcher'
import { Button } from '@/components/ui/button'
import { useProfiles } from '@/contexts/profiles'
import { cn } from '@/lib/utils'

const NAV = [
  { to: '/', label: 'Tableau de bord', icon: LayoutDashboard, end: true },
  { to: '/search', label: 'Recherche', icon: Search },
  { to: '/jobs', label: 'Offres', icon: Briefcase },
  { to: '/applications', label: 'Candidatures', icon: Sparkles },
  { to: '/profiles', label: 'Profils', icon: Users },
]

const TITLES: Record<string, string> = {
  '/': 'Tableau de bord',
  '/search': 'Recherche',
  '/jobs': 'Offres',
  '/applications': 'Candidatures',
  '/profiles': 'Mes profils',
  '/profile': 'Profil',
}

function NavItems({ onNavigate }: { onNavigate?: () => void }) {
  return (
    <nav className="flex flex-col gap-1">
      {NAV.map((n) => (
        <NavLink
          key={n.to}
          to={n.to}
          end={n.end}
          onClick={onNavigate}
          className={({ isActive }) =>
            cn(
              'flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors',
              isActive
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
            )
          }
        >
          <n.icon className="h-[18px] w-[18px]" />
          {n.label}
        </NavLink>
      ))}
    </nav>
  )
}

export function AppLayout() {
  const [mobileOpen, setMobileOpen] = useState(false)
  const { activeProfile } = useProfiles()
  const location = useLocation()

  const title = TITLES[location.pathname] ?? 'Workeo'

  return (
    <div className="flex min-h-screen bg-secondary/40">
      {/* Sidebar bureau */}
      <aside className="fixed inset-y-0 left-0 z-30 hidden w-64 flex-col border-r bg-background lg:flex">
        <div className="flex h-16 items-center border-b px-6">
          <Logo />
        </div>
        <div className="flex-1 p-4">
          <NavItems />
        </div>
        <div className="border-t p-4">
          <div className="rounded-xl bg-accent/60 p-3">
            <p className="text-xs font-medium text-accent-foreground">
              Profil actif
            </p>
            <p className="mt-1 truncate text-sm font-semibold">
              {activeProfile?.name ?? '—'}
            </p>
            <p className="truncate text-xs text-muted-foreground">
              {activeProfile?.headline || 'Aucun titre'}
            </p>
          </div>
        </div>
      </aside>

      {/* Drawer mobile */}
      <AnimatePresence>
        {mobileOpen && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={() => setMobileOpen(false)}
              className="fixed inset-0 z-40 bg-slate-900/40 backdrop-blur-sm lg:hidden"
            />
            <motion.aside
              initial={{ x: -300 }}
              animate={{ x: 0 }}
              exit={{ x: -300 }}
              transition={{ type: 'spring', stiffness: 320, damping: 34 }}
              className="fixed inset-y-0 left-0 z-50 flex w-72 flex-col border-r bg-background lg:hidden"
            >
              <div className="flex h-16 items-center justify-between border-b px-5">
                <Logo />
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setMobileOpen(false)}
                >
                  <X className="h-5 w-5" />
                </Button>
              </div>
              <div className="flex-1 p-4">
                <NavItems onNavigate={() => setMobileOpen(false)} />
              </div>
            </motion.aside>
          </>
        )}
      </AnimatePresence>

      {/* Contenu */}
      <div className="flex min-h-screen flex-1 flex-col lg:pl-64">
        <header className="sticky top-0 z-20 flex h-16 items-center gap-3 border-b bg-background/80 px-4 backdrop-blur-md sm:px-6">
          <Button
            variant="ghost"
            size="icon"
            className="lg:hidden"
            onClick={() => setMobileOpen(true)}
          >
            <Menu className="h-5 w-5" />
          </Button>
          <h1 className="hidden text-lg font-semibold tracking-tight sm:block">
            {title}
          </h1>
          <div className="ml-auto flex items-center gap-2">
            <ProfileSwitcher />
            <AccountMenu />
          </div>
        </header>

        <main className="flex-1 px-4 py-6 sm:px-6 lg:px-8">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
