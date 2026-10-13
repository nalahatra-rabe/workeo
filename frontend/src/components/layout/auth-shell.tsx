import { motion } from 'framer-motion'
import {
  Bot,
  FileSearch,
  ScanSearch,
  ShieldCheck,
  Sparkles,
  Target,
} from 'lucide-react'
import type { ReactNode } from 'react'

import { Logo } from '@/components/logo'
import { ThemeToggle } from '@/components/theme-toggle'

const FEATURES = [
  {
    icon: Bot,
    title: 'Une équipe d’agents IA',
    text: 'Analyse de CV, catégorisation et matching orchestrés automatiquement.',
  },
  {
    icon: ScanSearch,
    title: 'Recherche multi-sources',
    text: 'Arbeitnow, Remotive, RemoteOK et plus, agrégés en parallèle.',
  },
  {
    icon: Target,
    title: 'Matching ciblé',
    text: 'Un score de compatibilité expliqué pour chaque offre.',
  },
]

export function AuthShell({
  title,
  subtitle,
  children,
  footer,
}: {
  title: string
  subtitle: string
  children: ReactNode
  footer?: ReactNode
}) {
  return (
    <div className="flex min-h-screen bg-background">
      {/* Panneau de marque */}
      <div className="relative hidden w-[46%] overflow-hidden bg-primary lg:flex lg:flex-col xl:w-1/2">
        <div className="bg-grid absolute inset-0 opacity-[0.12]" />
        <div className="absolute -left-24 top-10 h-72 w-72 rounded-full bg-white/10 blur-2xl" />
        <div className="absolute -right-16 bottom-0 h-80 w-80 rounded-full bg-white/10 blur-2xl" />

        <div className="relative z-10 flex flex-1 flex-col p-12">
          <Logo
            to="/login"
            className="[&_span:first-child]:bg-white [&_span:first-child]:text-primary [&_span:last-child]:text-white"
          />

          <div className="mt-auto max-w-md">
            <motion.h1
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5 }}
              className="text-4xl font-bold leading-tight text-white"
            >
              Trouvez l’alternance qui vous ressemble.
            </motion.h1>
            <motion.p
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.08 }}
              className="mt-4 text-blue-100"
            >
              Workeo combine scraping d’offres et intelligence artificielle pour
              trouver, filtrer et prioriser les meilleures opportunités pour
              chaque profil.
            </motion.p>

            <div className="mt-10 space-y-5">
              {FEATURES.map((f, i) => (
                <motion.div
                  key={f.title}
                  initial={{ opacity: 0, x: -12 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: 0.15 + i * 0.1, duration: 0.4 }}
                  className="flex items-start gap-3"
                >
                  <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-white/15 text-white">
                    <f.icon className="h-5 w-5" />
                  </span>
                  <div>
                    <p className="font-semibold text-white">{f.title}</p>
                    <p className="text-sm text-white/75">{f.text}</p>
                  </div>
                </motion.div>
              ))}
            </div>
          </div>

          <div className="relative z-10 mt-12 flex items-center gap-4 text-xs text-white/70">
            <span className="inline-flex items-center gap-1.5">
              <ShieldCheck className="h-4 w-4" /> Données privées
            </span>
            <span className="inline-flex items-center gap-1.5">
              <Sparkles className="h-4 w-4" /> IA locale possible
            </span>
            <span className="inline-flex items-center gap-1.5">
              <FileSearch className="h-4 w-4" /> Multi-profils
            </span>
          </div>
        </div>
      </div>

      {/* Formulaire */}
      <div className="relative flex flex-1 items-center justify-center px-5 py-10">
        <div className="absolute right-4 top-4">
          <ThemeToggle />
        </div>
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
          className="w-full max-w-md"
        >
          <div className="mb-8 lg:hidden">
            <Logo />
          </div>
          <h2 className="text-2xl font-bold tracking-tight">{title}</h2>
          <p className="mt-1 text-sm text-muted-foreground">{subtitle}</p>
          <div className="mt-7">{children}</div>
          {footer && <div className="mt-6 text-sm">{footer}</div>}
        </motion.div>
      </div>
    </div>
  )
}
