import {
  ArrowRight,
  Briefcase,
  FileText,
  Loader2,
  Search,
  Sparkles,
  Target,
  TrendingUp,
} from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { JobCard } from '@/components/jobs/job-card'
import { JobDialog } from '@/components/jobs/job-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useAuth } from '@/contexts/auth'
import { useProfiles } from '@/contexts/profiles'
import { api } from '@/lib/api'
import type { ApplicationWithJob, ScoredJob, StatsResponse } from '@/lib/types'

export function DashboardPage() {
  const { user } = useAuth()
  const { activeProfile } = useProfiles()
  const navigate = useNavigate()

  const [stats, setStats] = useState<StatsResponse | null>(null)
  const [recommendations, setRecommendations] = useState<ScoredJob[]>([])
  const [applications, setApplications] = useState<ApplicationWithJob[]>([])
  const [loading, setLoading] = useState(true)
  const [selected, setSelected] = useState<ScoredJob | null>(null)

  useEffect(() => {
    if (!activeProfile) return
    let active = true
    setLoading(true)
    ;(async () => {
      try {
        const [s, recs, apps] = await Promise.all([
          api.stats(activeProfile.id),
          api.recommendations(activeProfile.id),
          api.listApplications(activeProfile.id),
        ])
        if (!active) return
        setStats(s)
        setRecommendations(recs)
        setApplications(apps)
      } catch {
        /* silencieux */
      } finally {
        if (active) setLoading(false)
      }
    })()
    return () => {
      active = false
    }
  }, [activeProfile])

  const savedIds = useMemo(
    () => new Set(applications.map((a) => a.job_id)),
    [applications],
  )

  async function saveJob(jobId: string) {
    if (!activeProfile) return
    try {
      await api.createApplication({
        profile_id: activeProfile.id,
        job_id: jobId,
        status: 'saved',
      })
      const list = await api.listApplications(activeProfile.id)
      setApplications(list)
      toast.success('Offre suivie', {
        description: 'Retrouvez-la dans vos candidatures.',
      })
    } catch (err) {
      toast.error('Impossible de suivre cette offre', {
        description: err instanceof Error ? err.message : undefined,
      })
    }
  }

  const firstName = user?.full_name?.split(' ')[0] ?? ''
  const topRecs = useMemo(() => recommendations.slice(0, 6), [recommendations])

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      {/* Bandeau */}
      <div className="overflow-hidden rounded-2xl bg-primary p-6 text-primary-foreground sm:p-8">
        <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="text-sm text-white/80">Bonjour {firstName} 👋</p>
            <h2 className="mt-1 text-2xl font-bold tracking-tight sm:text-3xl">
              {activeProfile?.name ?? 'Votre espace'}
            </h2>
            <p className="mt-1 max-w-xl text-sm text-white/80">
              {activeProfile?.headline
                ? activeProfile.headline
                : 'Lancez une recherche pour découvrir des offres adaptées à votre profil.'}
            </p>
          </div>
          <Button
            size="lg"
            variant="secondary"
            className="bg-white text-primary hover:bg-white/90"
            onClick={() => navigate('/search')}
          >
            <Search className="h-4 w-4" /> Lancer une recherche
          </Button>
        </div>
      </div>

      {/* Statistiques */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          icon={TrendingUp}
          label="Offres suivies"
          value={stats?.applications ?? applications.length}
          loading={loading}
        />
        <StatCard
          icon={Target}
          label="Offres compatibles"
          value={stats?.matches ?? recommendations.length}
          loading={loading}
        />
        <StatCard
          icon={Search}
          label="Recherches"
          value={stats?.searches ?? 0}
          loading={loading}
        />
        <StatCard
          icon={Briefcase}
          label="Offres en base"
          value={stats?.jobs ?? 0}
          loading={loading}
        />
      </div>

      {/* Recommandations */}
      <Card>
        <CardContent className="p-6">
          <div className="mb-5 flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="flex h-9 w-9 items-center justify-center rounded-lg bg-accent text-primary">
                <Sparkles className="h-4 w-4" />
              </span>
              <div>
                <h3 className="font-semibold">Recommandations</h3>
                <p className="text-xs text-muted-foreground">
                  Les meilleures offres pour « {activeProfile?.name} »
                </p>
              </div>
            </div>
            {recommendations.length > 0 && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => navigate('/jobs')}
              >
                Tout voir <ArrowRight className="h-4 w-4" />
              </Button>
            )}
          </div>

          {loading ? (
            <div className="grid gap-4 md:grid-cols-2">
              {[0, 1, 2, 3].map((i) => (
                <Skeleton key={i} className="h-48 rounded-xl" />
              ))}
            </div>
          ) : topRecs.length === 0 ? (
            <EmptyRecommendations onSearch={() => navigate('/search')} />
          ) : (
            <div className="grid gap-4 md:grid-cols-2">
              {topRecs.map((r) => (
                <JobCard
                  key={r.id}
                  job={r}
                  score={r.score}
                  reasons={r.reasons}
                  saved={savedIds.has(r.id)}
                  onSave={() => saveJob(r.id)}
                  onOpen={() => setSelected(r)}
                />
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <JobDialog
        open={!!selected}
        onOpenChange={(o) => !o && setSelected(null)}
        job={selected}
        score={selected?.score}
        reasons={selected?.reasons}
        saved={!!selected && savedIds.has(selected.id)}
        onSave={() => selected && saveJob(selected.id)}
      />
    </div>
  )
}

function StatCard({
  icon: Icon,
  label,
  value,
  loading,
}: {
  icon: React.ComponentType<{ className?: string }>
  label: string
  value: number
  loading: boolean
}) {
  return (
    <Card>
      <CardContent className="flex items-center gap-4 p-5">
        <span className="flex h-11 w-11 items-center justify-center rounded-xl bg-accent text-primary">
          <Icon className="h-5 w-5" />
        </span>
        <div>
          <p className="text-xs font-medium text-muted-foreground">{label}</p>
          {loading ? (
            <Loader2 className="mt-1 h-5 w-5 animate-spin text-muted-foreground" />
          ) : (
            <p className="text-2xl font-bold tabular-nums">{value}</p>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function EmptyRecommendations({ onSearch }: { onSearch: () => void }) {
  return (
    <div className="flex flex-col items-center justify-center rounded-xl border border-dashed py-14 text-center">
      <span className="flex h-14 w-14 items-center justify-center rounded-full bg-accent text-primary">
        <FileText className="h-6 w-6" />
      </span>
      <p className="mt-4 font-medium">Aucune recommandation pour l’instant</p>
      <p className="mt-1 max-w-sm text-sm text-muted-foreground">
        Lancez une recherche pour collecter des offres et laisser les agents IA
        calculer vos meilleurs matchs.
      </p>
      <Button className="mt-5" onClick={onSearch}>
        <Search className="h-4 w-4" /> Lancer une recherche
      </Button>
    </div>
  )
}
