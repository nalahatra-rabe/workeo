import { AnimatePresence, motion } from 'framer-motion'
import {
  Bot,
  Brain,
  CheckCircle2,
  ChevronRight,
  Filter,
  Layers,
  Loader2,
  LocateFixed,
  MapPin,
  Radar,
  Search,
  Sparkles,
  Target,
  Wifi,
} from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { toast } from 'sonner'

import { JobCard } from '@/components/jobs/job-card'
import { JobDialog } from '@/components/jobs/job-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useProfiles } from '@/contexts/profiles'
import { api } from '@/lib/api'
import type { ScoredJob, SourcesResponse } from '@/lib/types'
import { cn } from '@/lib/utils'

const PIPELINE = [
  { icon: Brain, label: 'Analyse du profil candidat' },
  { icon: Target, label: 'Génération des mots-clés ciblés' },
  { icon: Radar, label: 'Interrogation des sources en parallèle' },
  { icon: Layers, label: 'Catégorisation des offres (agent IA)' },
  { icon: Bot, label: 'Calcul des scores de compatibilité' },
  { icon: Sparkles, label: 'Classement des meilleures offres' },
]

export function SearchPage() {
  const { activeProfile } = useProfiles()

  const [query, setQuery] = useState('')
  const [location, setLocation] = useState('')
  const [remoteOnly, setRemoteOnly] = useState(false)
  const [contract, setContract] = useState('any')
  const [sources, setSources] = useState<string[]>([])
  const [availableSources, setAvailableSources] =
    useState<SourcesResponse | null>(null)

  const [searching, setSearching] = useState(false)
  const [step, setStep] = useState(-1)
  const [results, setResults] = useState<ScoredJob[]>([])
  const [errors, setErrors] = useState<string[]>([])
  const [ran, setRan] = useState(false)
  const [selected, setSelected] = useState<ScoredJob | null>(null)
  const [savedIds, setSavedIds] = useState<Set<string>>(new Set())
  const [locating, setLocating] = useState(false)

  useEffect(() => {
    setLocation(activeProfile?.location ?? '')
  }, [activeProfile])

  useEffect(() => {
    api
      .sources()
      .then((s) => {
        setAvailableSources(s)
        setSources(s.active_default)
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

  const contractOptions = activeProfile?.contract_types?.length
    ? activeProfile.contract_types
    : ['Alternance', 'Stage', 'CDI', 'CDD', 'Freelance']

  const canSearch = useMemo(() => !!activeProfile && !searching, [
    activeProfile,
    searching,
  ])

  function toggleSource(name: string) {
    setSources((prev) =>
      prev.includes(name) ? prev.filter((s) => s !== name) : [...prev, name],
    )
  }

  /** Géolocalisation : remplit la localisation via reverse geocoding. */
  function useMyLocation() {
    if (!('geolocation' in navigator)) {
      toast.error('Géolocalisation non supportée par ce navigateur.')
      return
    }
    setLocating(true)
    navigator.geolocation.getCurrentPosition(
      async (pos) => {
        const { latitude, longitude } = pos.coords
        const fallback = `${latitude.toFixed(3)}, ${longitude.toFixed(3)}`
        try {
          const res = await fetch(
            `https://nominatim.openstreetmap.org/reverse?format=jsonv2&lat=${latitude}&lon=${longitude}&accept-language=fr`,
          )
          if (res.ok) {
            const data = await res.json()
            const a = data.address ?? {}
            const city =
              a.city || a.town || a.village || a.municipality || a.county
            setLocation([city, a.country].filter(Boolean).join(', ') || fallback)
          } else {
            setLocation(fallback)
          }
        } catch {
          setLocation(fallback)
        } finally {
          setLocating(false)
          toast.success('Position détectée')
        }
      },
      () => {
        setLocating(false)
        toast.error('Localisation refusée ou indisponible.')
      },
      { timeout: 8000 },
    )
  }

  /** Notification navigateur à la fin d'une recherche. */
  function notifySearchDone(total: number) {
    if (!('Notification' in window)) return
    if (Notification.permission !== 'granted') return
    try {
      new Notification('Workeo — recherche terminée', {
        body: `${total} offre(s) classée(s) par les agents.`,
      })
    } catch {
      /* ignoré */
    }
  }

  async function runSearch(e?: React.FormEvent) {
    e?.preventDefault()
    if (!activeProfile) return
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission().catch(() => {})
    }
    setSearching(true)
    setRan(false)
    setResults([])
    setErrors([])
    setStep(0)
    const timer = setInterval(
      () => setStep((s) => (s < PIPELINE.length - 1 ? s + 1 : s)),
      600,
    )
    try {
      const res = await api.search({
        profile_id: activeProfile.id,
        query: query.trim() || undefined,
        location: location.trim() || undefined,
        remote_only: remoteOnly || undefined,
        contract_type: contract === 'any' ? undefined : contract,
        sources: sources.length ? sources : undefined,
      })
      setStep(PIPELINE.length - 1)
      setResults(res.results)
      setErrors(res.source_errors)
      setRan(true)
      if (res.results.length === 0) {
        toast.info('Aucune offre trouvée', {
          description: 'Essayez d’élargir vos critères.',
        })
      } else {
        toast.success(`${res.total} offres classées`, {
          description: 'Les agents ont terminé leur analyse.',
        })
        notifySearchDone(res.total)
      }
    } catch (err) {
      toast.error('Recherche impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    } finally {
      clearInterval(timer)
      setSearching(false)
      setStep(-1)
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
      toast.error('Action impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    }
  }

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <div>
        <h2 className="text-xl font-bold tracking-tight">
          Recherche multi-agents
        </h2>
        <p className="text-sm text-muted-foreground">
          Les agents IA collectent, catégorisent et classent les offres pour «{' '}
          {activeProfile?.name} ».
        </p>
      </div>

      {/* Formulaire */}
      <Card>
        <CardContent className="p-6">
          <form onSubmit={runSearch} className="space-y-5">
            <div className="grid gap-5 md:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="query">Mots-clés</Label>
                <div className="relative">
                  <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    id="query"
                    className="pl-9"
                    placeholder={
                      activeProfile?.target_roles?.[0] ||
                      'Ex : développeur rust alternance'
                    }
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                  />
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="location">Localisation</Label>
                <div className="relative">
                  <MapPin className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    id="location"
                    placeholder="Paris, France"
                    className="pl-9 pr-10"
                    value={location}
                    onChange={(e) => setLocation(e.target.value)}
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    onClick={useMyLocation}
                    disabled={locating}
                    aria-label="Utiliser ma position"
                    title="Utiliser ma position"
                    className="absolute right-1 top-1/2 h-7 w-7 -translate-y-1/2 text-muted-foreground"
                  >
                    {locating ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <LocateFixed className="h-4 w-4" />
                    )}
                  </Button>
                </div>
              </div>
              <div className="space-y-2">
                <Label>Type de contrat</Label>
                <Select value={contract} onValueChange={setContract}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="any">Tous les contrats</SelectItem>
                    {contractOptions.map((c) => (
                      <SelectItem key={c} value={c}>
                        {c}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-end">
                <button
                  type="button"
                  onClick={() => setRemoteOnly((v) => !v)}
                  className={cn(
                    'flex h-10 w-full items-center gap-2 rounded-lg border px-3.5 text-sm font-medium transition-colors',
                    remoteOnly
                      ? 'border-primary bg-accent text-accent-foreground'
                      : 'border-input hover:bg-accent/50',
                  )}
                >
                  <Wifi className="h-4 w-4" />
                  Télétravail uniquement
                </button>
              </div>
            </div>

            {availableSources && (
              <div className="space-y-2">
                <Label>Sources</Label>
                <div className="flex flex-wrap gap-2">
                  {availableSources.sources.map((s) => {
                    const active = sources.includes(s.name)
                    return (
                      <button
                        key={s.name}
                        type="button"
                        onClick={() => toggleSource(s.name)}
                        className={cn(
                          'rounded-full border px-3.5 py-1.5 text-sm font-medium transition-colors',
                          active
                            ? 'border-primary bg-primary text-primary-foreground'
                            : 'border-input bg-background hover:bg-accent',
                        )}
                      >
                        {s.label}
                      </button>
                    )
                  })}
                </div>
              </div>
            )}

            <Button type="submit" size="lg" disabled={!canSearch}>
              {searching ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Search className="h-4 w-4" />
              )}
              Lancer la recherche
            </Button>
          </form>
        </CardContent>
      </Card>

      {/* Pipeline */}
      <AnimatePresence>
        {searching && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="overflow-hidden"
          >
            <Card>
              <CardContent className="p-6">
                <div className="mb-4 flex items-center gap-2">
                  <Bot className="h-4 w-4 text-primary" />
                  <p className="text-sm font-semibold">
                    Les agents travaillent…
                  </p>
                </div>
                <div className="space-y-1">
                  {PIPELINE.map((p, i) => {
                    const done = i < step
                    const current = i === step
                    return (
                      <motion.div
                        key={p.label}
                        initial={{ opacity: 0.4 }}
                        animate={{ opacity: done || current ? 1 : 0.4 }}
                        className={cn(
                          'flex items-center gap-3 rounded-lg px-3 py-2',
                          current && 'bg-accent',
                        )}
                      >
                        <span
                          className={cn(
                            'flex h-7 w-7 items-center justify-center rounded-full',
                            done
                              ? 'bg-emerald-100 text-emerald-600'
                              : current
                                ? 'bg-primary text-primary-foreground'
                                : 'bg-muted text-muted-foreground',
                          )}
                        >
                          {done ? (
                            <CheckCircle2 className="h-4 w-4" />
                          ) : current ? (
                            <Loader2 className="h-4 w-4 animate-spin" />
                          ) : (
                            <p.icon className="h-4 w-4" />
                          )}
                        </span>
                        <span className="text-sm">{p.label}</span>
                      </motion.div>
                    )
                  })}
                </div>
              </CardContent>
            </Card>
          </motion.div>
        )}
      </AnimatePresence>

      {errors.length > 0 && !searching && (
        <div className="rounded-xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-800">
          <p className="font-medium">Certaines sources n’ont pas répondu :</p>
          <ul className="mt-1 list-inside list-disc">
            {errors.map((e) => (
              <li key={e}>{e}</li>
            ))}
          </ul>
        </div>
      )}

      {/* Résultats */}
      {ran && !searching && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Filter className="h-4 w-4 text-muted-foreground" />
              <p className="text-sm text-muted-foreground">
                {results.length} offre(s) classée(s) par pertinence
              </p>
            </div>
          </div>
          {results.length === 0 ? (
            <div className="flex flex-col items-center justify-center rounded-xl border border-dashed py-14 text-center">
              <span className="flex h-14 w-14 items-center justify-center rounded-full bg-accent text-primary">
                <Search className="h-6 w-6" />
              </span>
              <p className="mt-4 font-medium">Aucun résultat</p>
              <p className="mt-1 max-w-sm text-sm text-muted-foreground">
                Essayez d’élargir la localisation ou de retirer le filtre
                télétravail.
              </p>
            </div>
          ) : (
            <div className="grid gap-4 md:grid-cols-2">
              {results.map((r) => (
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
        </div>
      )}

      {!ran && !searching && (
        <div className="flex flex-col items-center justify-center rounded-xl border border-dashed py-16 text-center">
          <span className="flex h-14 w-14 items-center justify-center rounded-full bg-accent text-primary">
            <ChevronRight className="h-6 w-6" />
          </span>
          <p className="mt-4 font-medium">Prêt à explorer le marché</p>
          <p className="mt-1 max-w-sm text-sm text-muted-foreground">
            Configurez vos critères puis lancez la recherche multi-agents.
          </p>
        </div>
      )}

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
