import { Filter, Loader2, Search, ToggleLeft, ToggleRight } from 'lucide-react'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'

import { JobCard } from '@/components/jobs/job-card'
import { JobDialog } from '@/components/jobs/job-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Skeleton } from '@/components/ui/skeleton'
import { useProfiles } from '@/contexts/profiles'
import { api } from '@/lib/api'
import type { Job } from '@/lib/types'

const PAGE = 24

export function JobsPage() {
  const { activeProfile } = useProfiles()

  const [jobs, setJobs] = useState<Job[]>([])
  const [loading, setLoading] = useState(true)
  const [loadingMore, setLoadingMore] = useState(false)
  const [hasMore, setHasMore] = useState(true)
  const [categories, setCategories] = useState<string[]>([])
  const [savedIds, setSavedIds] = useState<Set<string>>(new Set())
  const [selected, setSelected] = useState<Job | null>(null)

  const [query, setQuery] = useState('')
  const [remote, setRemote] = useState(false)
  const [contract, setContract] = useState('any')
  const [category, setCategory] = useState('any')

  const contractOptions = activeProfile?.contract_types?.length
    ? activeProfile.contract_types
    : ['Alternance', 'Stage', 'CDI', 'CDD', 'Freelance']

  // Chargement initial + à chaque changement de filtre (avec léger debounce).
  useEffect(() => {
    let active = true
    setLoading(true)
    const timer = setTimeout(async () => {
      try {
        const page = await api.listJobs({
          q: query.trim() || undefined,
          remote: remote || undefined,
          contract: contract === 'any' ? undefined : contract,
          category: category === 'any' ? undefined : category,
          limit: PAGE,
          offset: 0,
        })
        if (!active) return
        setJobs(page)
        setHasMore(page.length === PAGE)
      } catch (err) {
        toast.error('Chargement impossible', {
          description: err instanceof Error ? err.message : undefined,
        })
      } finally {
        if (active) setLoading(false)
      }
    }, 250)
    return () => {
      active = false
      clearTimeout(timer)
    }
  }, [query, remote, contract, category])

  useEffect(() => {
    api
      .listJobs({ limit: 200 })
      .then((all) => {
        const cats = Array.from(
          new Set(all.map((j) => j.category).filter(Boolean) as string[]),
        ).sort()
        setCategories(cats)
      })
      .catch(() => {})
  }, [])

  useEffect(() => {
    if (!activeProfile) return
    api
      .listApplications(activeProfile.id)
      .then((apps) => setSavedIds(new Set(apps.map((a) => a.job_id))))
      .catch(() => {})
  }, [activeProfile])

  async function loadMore() {
    setLoadingMore(true)
    try {
      const page = await api.listJobs({
        q: query.trim() || undefined,
        remote: remote || undefined,
        contract: contract === 'any' ? undefined : contract,
        category: category === 'any' ? undefined : category,
        limit: PAGE,
        offset: jobs.length,
      })
      setJobs((prev) => [...prev, ...page])
      setHasMore(page.length === PAGE)
    } catch (err) {
      toast.error('Chargement impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    } finally {
      setLoadingMore(false)
    }
  }

  async function saveJob(jobId: string) {
    if (!activeProfile) return
    try {
      await api.createApplication({
        profile_id: activeProfile.id,
        job_id: jobId,
        status: 'saved',
      })
      setSavedIds((prev) => new Set(prev).add(jobId))
      toast.success('Offre suivie')
    } catch (err) {
      toast.error('Impossible de suivre cette offre', {
        description: err instanceof Error ? err.message : undefined,
      })
    }
  }

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <Card>
        <CardContent className="grid gap-4 p-5 md:grid-cols-[1fr_auto_auto_auto_auto]">
          <div className="relative">
            <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Rechercher un poste, une entreprise…"
              className="pl-9"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>

          <Select value={contract} onValueChange={setContract}>
            <SelectTrigger className="md:w-40">
              <SelectValue placeholder="Contrat" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="any">Tous contrats</SelectItem>
              {contractOptions.map((c) => (
                <SelectItem key={c} value={c}>
                  {c}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select value={category} onValueChange={setCategory}>
            <SelectTrigger className="md:w-48">
              <SelectValue placeholder="Catégorie" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="any">Toutes catégories</SelectItem>
              {categories.map((c) => (
                <SelectItem key={c} value={c}>
                  {c}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <button
            type="button"
            onClick={() => setRemote((v) => !v)}
            className="inline-flex h-10 items-center gap-2 rounded-lg border border-input px-3 text-sm font-medium transition-colors hover:bg-accent"
          >
            {remote ? (
              <ToggleRight className="h-5 w-5 text-primary" />
            ) : (
              <ToggleLeft className="h-5 w-5 text-muted-foreground" />
            )}
            Télétravail
          </button>

          <div className="flex items-center justify-center gap-2 px-2 text-sm text-muted-foreground">
            <Filter className="h-4 w-4" />
            {jobs.length}
          </div>
        </CardContent>
      </Card>

      {loading ? (
        <div className="grid gap-4 md:grid-cols-2">
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className="h-48 rounded-xl" />
          ))}
        </div>
      ) : jobs.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-xl border border-dashed py-16 text-center">
          <span className="flex h-14 w-14 items-center justify-center rounded-full bg-accent text-primary">
            <Search className="h-6 w-6" />
          </span>
          <p className="mt-4 font-medium">Aucune offre trouvée</p>
          <p className="mt-1 max-w-sm text-sm text-muted-foreground">
            Lancez une recherche pour alimenter la base, ou ajustez vos filtres.
          </p>
        </div>
      ) : (
        <>
          <div className="grid gap-4 md:grid-cols-2">
            {jobs.map((j) => (
              <JobCard
                key={j.id}
                job={j}
                saved={savedIds.has(j.id)}
                onSave={() => saveJob(j.id)}
                onOpen={() => setSelected(j)}
              />
            ))}
          </div>
          {hasMore && (
            <div className="flex justify-center pt-2">
              <Button
                variant="outline"
                onClick={loadMore}
                disabled={loadingMore}
              >
                {loadingMore && <Loader2 className="h-4 w-4 animate-spin" />}
                Charger plus d’offres
              </Button>
            </div>
          )}
        </>
      )}

      <JobDialog
        open={!!selected}
        onOpenChange={(o) => !o && setSelected(null)}
        job={selected}
        saved={!!selected && savedIds.has(selected.id)}
        onSave={() => selected && saveJob(selected.id)}
      />
    </div>
  )
}
