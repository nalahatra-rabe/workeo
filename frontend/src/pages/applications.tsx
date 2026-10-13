import { motion } from 'framer-motion'
import {
  Building2,
  ExternalLink,
  MoreHorizontal,
  Send,
  Trash2,
} from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Skeleton } from '@/components/ui/skeleton'
import { useProfiles } from '@/contexts/profiles'
import { api } from '@/lib/api'
import type { ApplicationStatus, ApplicationWithJob } from '@/lib/types'
import { cn } from '@/lib/utils'

const COLUMNS: {
  key: ApplicationStatus
  label: string
  dot: string
}[] = [
  { key: 'saved', label: 'À étudier', dot: 'bg-slate-400' },
  { key: 'applied', label: 'Candidatures', dot: 'bg-blue-500' },
  { key: 'interview', label: 'Entretiens', dot: 'bg-amber-500' },
  { key: 'offer', label: 'Offres', dot: 'bg-emerald-500' },
  { key: 'rejected', label: 'Refus', dot: 'bg-rose-400' },
]

export function ApplicationsPage() {
  const { activeProfile } = useProfiles()
  const [apps, setApps] = useState<ApplicationWithJob[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (!activeProfile) return
    let active = true
    setLoading(true)
    ;(async () => {
      try {
        const list = await api.listApplications(activeProfile.id)
        if (active) setApps(list)
      } catch (err) {
        toast.error('Chargement impossible', {
          description: err instanceof Error ? err.message : undefined,
        })
      } finally {
        if (active) setLoading(false)
      }
    })()
    return () => {
      active = false
    }
  }, [activeProfile])

  const grouped = useMemo(() => {
    const map: Record<string, ApplicationWithJob[]> = {}
    for (const c of COLUMNS) map[c.key] = []
    for (const a of apps) {
      ;(map[a.status] ??= []).push(a)
    }
    return map
  }, [apps])

  async function move(app: ApplicationWithJob, status: ApplicationStatus) {
    const prev = apps
    setApps((list) =>
      list.map((a) => (a.id === app.id ? { ...a, status } : a)),
    )
    try {
      await api.updateApplication(app.id, { status })
    } catch (err) {
      setApps(prev)
      toast.error('Mise à jour impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    }
  }

  async function remove(app: ApplicationWithJob) {
    const prev = apps
    setApps((list) => list.filter((a) => a.id !== app.id))
    try {
      await api.deleteApplication(app.id)
      toast.success('Candidature supprimée')
    } catch (err) {
      setApps(prev)
      toast.error('Suppression impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    }
  }

  if (loading) {
    return (
      <div className="grid gap-4 lg:grid-cols-5">
        {COLUMNS.map((c) => (
          <Skeleton key={c.key} className="h-64 rounded-xl" />
        ))}
      </div>
    )
  }

  if (apps.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center rounded-xl border border-dashed py-20 text-center">
        <span className="flex h-14 w-14 items-center justify-center rounded-full bg-accent text-primary">
          <Send className="h-6 w-6" />
        </span>
        <p className="mt-4 font-medium">Aucune candidature suivie</p>
        <p className="mt-1 max-w-sm text-sm text-muted-foreground">
          Suivez des offres depuis les recommandations ou l’explorateur pour les
          organiser ici.
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        {apps.length} candidature(s) — déplacez-les au fil de vos avancées.
      </p>
      <div className="grid gap-4 overflow-x-auto lg:grid-cols-5">
        {COLUMNS.map((col) => (
          <div key={col.key} className="flex flex-col gap-3">
            <div className="flex items-center justify-between px-1">
              <div className="flex items-center gap-2">
                <span className={cn('h-2.5 w-2.5 rounded-full', col.dot)} />
                <p className="text-sm font-semibold">{col.label}</p>
              </div>
              <span className="rounded-md bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">
                {grouped[col.key]?.length ?? 0}
              </span>
            </div>

            <div className="flex min-h-[120px] flex-col gap-3 rounded-xl bg-secondary/60 p-2">
              {grouped[col.key]?.map((app) => (
                <motion.div
                  key={app.id}
                  layout
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                >
                  <Card className="border-transparent shadow-sm transition-shadow hover:shadow-md">
                    <CardContent className="p-4">
                      <div className="flex items-start justify-between gap-2">
                        <div className="min-w-0">
                          <p className="truncate text-sm font-semibold">
                            {app.job?.title ?? 'Offre supprimée'}
                          </p>
                          <p className="mt-0.5 flex items-center gap-1 truncate text-xs text-muted-foreground">
                            <Building2 className="h-3 w-3" />
                            {app.job?.company ?? '—'}
                          </p>
                        </div>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-7 w-7 shrink-0"
                            >
                              <MoreHorizontal className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuLabel>Déplacer vers</DropdownMenuLabel>
                            {COLUMNS.filter((c) => c.key !== app.status).map(
                              (c) => (
                                <DropdownMenuItem
                                  key={c.key}
                                  onClick={() => move(app, c.key)}
                                >
                                  <span
                                    className={cn(
                                      'h-2 w-2 rounded-full',
                                      c.dot,
                                    )}
                                  />
                                  {c.label}
                                </DropdownMenuItem>
                              ),
                            )}
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              className="text-destructive focus:text-destructive"
                              onClick={() => remove(app)}
                            >
                              <Trash2 className="h-4 w-4" /> Supprimer
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </div>

                      {app.job?.location && (
                        <Badge
                          variant="secondary"
                          className="mt-3 font-normal"
                        >
                          {app.job.location}
                        </Badge>
                      )}

                      {app.job?.url && (
                        <a
                          href={app.job.url}
                          target="_blank"
                          rel="noreferrer"
                          className="mt-3 inline-flex items-center gap-1 text-xs font-medium text-primary hover:underline"
                        >
                          Voir l’offre <ExternalLink className="h-3 w-3" />
                        </a>
                      )}
                    </CardContent>
                  </Card>
                </motion.div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
