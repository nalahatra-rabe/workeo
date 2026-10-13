import { AnimatePresence, motion } from 'framer-motion'
import {
  ArrowLeft,
  ArrowRight,
  BadgeCheck,
  Brain,
  Briefcase,
  CheckCircle2,
  FileText,
  Languages as LanguagesIcon,
  Loader2,
  MapPin,
  Rocket,
  ScanLine,
  Sparkles,
  Target,
  UploadCloud,
  Wand2,
} from 'lucide-react'
import { useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { Logo } from '@/components/logo'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useAuth } from '@/contexts/auth'
import { useProfiles } from '@/contexts/profiles'
import type { Profile } from '@/lib/types'
import { cn } from '@/lib/utils'

const CONTRACTS = ['Alternance', 'Stage', 'CDI', 'CDD', 'Freelance']
const REMOTE = [
  { value: 'any', label: 'Peu importe' },
  { value: 'remote', label: '100 % télétravail' },
  { value: 'hybrid', label: 'Hybride' },
  { value: 'on-site', label: 'Sur site' },
]

const AGENT_STEPS = [
  { icon: FileText, label: 'Lecture du document' },
  { icon: ScanLine, label: 'Extraction des informations' },
  { icon: Brain, label: 'Analyse par l’agent IA' },
  { icon: Target, label: 'Catégorisation du profil' },
  { icon: Sparkles, label: 'Génération des mots-clés' },
]

const STEP_TITLES = ['Bienvenue', 'Votre profil', 'Votre CV', 'Récapitulatif']

const container = {
  hidden: {},
  show: { transition: { staggerChildren: 0.06 } },
}
const item = {
  hidden: { opacity: 0, y: 12 },
  show: { opacity: 1, y: 0 },
}

export function OnboardingPage() {
  const { user } = useAuth()
  const { createProfile, uploadCv, analyze } = useProfiles()
  const navigate = useNavigate()

  const [step, setStep] = useState(0)
  const [saving, setSaving] = useState(false)

  const [name, setName] = useState('')
  const [fullName, setFullName] = useState(user?.full_name ?? '')
  const [headline, setHeadline] = useState('')
  const [location, setLocation] = useState('')
  const [remote, setRemote] = useState('any')
  const [contracts, setContracts] = useState<string[]>(['Alternance'])

  const [profile, setProfile] = useState<Profile | null>(null)

  const [file, setFile] = useState<File | null>(null)
  const [analyzing, setAnalyzing] = useState(false)
  const [agentStep, setAgentStep] = useState(-1)

  function toggleContract(c: string) {
    setContracts((prev) =>
      prev.includes(c) ? prev.filter((x) => x !== c) : [...prev, c],
    )
  }

  async function submitProfile() {
    if (!fullName.trim()) {
      toast.error('Le nom complet est requis.')
      return
    }
    setSaving(true)
    try {
      const p = await createProfile({
        name: name.trim() || headline.trim() || 'Mon profil',
        full_name: fullName.trim(),
        email: user?.email,
        location: location.trim() || undefined,
        headline: headline.trim() || undefined,
        remote_preference: remote,
        contract_types: contracts,
        is_default: true,
      })
      setProfile(p)
      setStep(2)
    } catch (err) {
      toast.error('Création du profil impossible', {
        description: err instanceof Error ? err.message : 'Réessayez.',
      })
    } finally {
      setSaving(false)
    }
  }

  async function startAnalysis() {
    if (!profile || !file) {
      setStep(3)
      return
    }
    setAnalyzing(true)
    setAgentStep(0)
    const timer = setInterval(() => {
      setAgentStep((s) => (s < AGENT_STEPS.length - 1 ? s + 1 : s))
    }, 700)
    try {
      const updated = await uploadCv(profile.id, file)
      setProfile(updated)
      setAgentStep(AGENT_STEPS.length - 1)
      await new Promise((r) => setTimeout(r, 450))
      toast.success('CV analysé !', {
        description: `${updated.skills.length} compétences détectées.`,
      })
      setStep(3)
    } catch (err) {
      toast.error('Analyse impossible', {
        description: err instanceof Error ? err.message : 'Réessayez.',
      })
      setStep(3)
    } finally {
      clearInterval(timer)
      setAnalyzing(false)
      setAgentStep(-1)
    }
  }

  async function reanalyze() {
    if (!profile) return
    setAnalyzing(true)
    setAgentStep(0)
    const timer = setInterval(
      () => setAgentStep((s) => (s < AGENT_STEPS.length - 1 ? s + 1 : s)),
      600,
    )
    try {
      const updated = await analyze(profile.id)
      setProfile(updated)
      toast.success('Profil réanalysé.')
    } catch {
      toast.error('Analyse impossible.')
    } finally {
      clearInterval(timer)
      setAnalyzing(false)
      setAgentStep(-1)
    }
  }

  const progress = (step / (STEP_TITLES.length - 1)) * 100

  return (
    <div className="relative flex min-h-screen flex-col bg-background">
      <header className="border-b">
        <div className="mx-auto flex h-16 w-full max-w-4xl items-center justify-between px-5">
          <Logo to="/onboarding" />
          <div className="hidden items-center gap-1.5 sm:flex">
            {STEP_TITLES.map((t, i) => (
              <div key={t} className="flex items-center gap-1.5">
                <span
                  className={cn(
                    'flex h-6 w-6 items-center justify-center rounded-full text-xs font-semibold transition-colors',
                    i <= step
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted text-muted-foreground',
                  )}
                >
                  {i < step ? <CheckCircle2 className="h-3.5 w-3.5" /> : i + 1}
                </span>
                <span
                  className={cn(
                    'text-xs font-medium',
                    i <= step ? 'text-foreground' : 'text-muted-foreground',
                  )}
                >
                  {t}
                </span>
                {i < STEP_TITLES.length - 1 && (
                  <span className="mx-1 h-px w-6 bg-border" />
                )}
              </div>
            ))}
          </div>
        </div>
        <div className="h-1 w-full bg-muted">
          <motion.div
            className="h-full bg-primary"
            animate={{ width: `${Math.max(progress, 8)}%` }}
            transition={{ type: 'spring', stiffness: 120, damping: 20 }}
          />
        </div>
      </header>

      <main className="mx-auto flex w-full max-w-3xl flex-1 items-center px-5 py-10">
        <AnimatePresence mode="wait">
          {step === 0 && (
            <Step key="welcome">
              <WelcomeStep name={user?.full_name} onNext={() => setStep(1)} />
            </Step>
          )}
          {step === 1 && (
            <Step key="profile">
              <form
                onSubmit={(e) => {
                  e.preventDefault()
                  submitProfile()
                }}
                className="space-y-6"
              >
                <StepHeading
                  icon={Briefcase}
                  title="Créons votre profil"
                  subtitle="Ces informations guident la recherche. Vous pourrez les modifier plus tard."
                />

                <div className="grid gap-5 sm:grid-cols-2">
                  <div className="space-y-2">
                    <Label htmlFor="name">Nom du profil</Label>
                    <Input
                      id="name"
                      placeholder="Ex : Alternance Dev Web"
                      value={name}
                      onChange={(e) => setName(e.target.value)}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="fullName">Nom complet *</Label>
                    <Input
                      id="fullName"
                      placeholder="Alice Martin"
                      value={fullName}
                      onChange={(e) => setFullName(e.target.value)}
                      required
                    />
                  </div>
                  <div className="space-y-2 sm:col-span-2">
                    <Label htmlFor="headline">Titre / objectif</Label>
                    <Input
                      id="headline"
                      placeholder="Ex : Développeur full-stack en alternance"
                      value={headline}
                      onChange={(e) => setHeadline(e.target.value)}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="location">Localisation</Label>
                    <div className="relative">
                      <MapPin className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                      <Input
                        id="location"
                        placeholder="Paris, France"
                        className="pl-9"
                        value={location}
                        onChange={(e) => setLocation(e.target.value)}
                      />
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Label>Télétravail souhaité</Label>
                    <Select value={remote} onValueChange={setRemote}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {REMOTE.map((r) => (
                          <SelectItem key={r.value} value={r.value}>
                            {r.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label>Types de contrat</Label>
                  <div className="flex flex-wrap gap-2">
                    {CONTRACTS.map((c) => {
                      const active = contracts.includes(c)
                      return (
                        <button
                          key={c}
                          type="button"
                          onClick={() => toggleContract(c)}
                          className={cn(
                            'rounded-full border px-3.5 py-1.5 text-sm font-medium transition-colors',
                            active
                              ? 'border-primary bg-primary text-primary-foreground'
                              : 'border-input bg-background hover:bg-accent',
                          )}
                        >
                          {c}
                        </button>
                      )
                    })}
                  </div>
                </div>

                <div className="flex items-center justify-between pt-2">
                  <Button
                    type="button"
                    variant="ghost"
                    onClick={() => setStep(0)}
                  >
                    <ArrowLeft className="h-4 w-4" /> Retour
                  </Button>
                  <Button type="submit" disabled={saving}>
                    {saving ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <ArrowRight className="h-4 w-4" />
                    )}
                    Continuer
                  </Button>
                </div>
              </form>
            </Step>
          )}
          {step === 2 && (
            <Step key="cv">
              <CvStep
                file={file}
                setFile={setFile}
                analyzing={analyzing}
                agentStep={agentStep}
                onBack={() => setStep(1)}
                onAnalyze={startAnalysis}
                onSkip={() => setStep(3)}
              />
            </Step>
          )}
          {step === 3 && (
            <Step key="review">
              <ReviewStep
                profile={profile}
                analyzing={analyzing}
                agentStep={agentStep}
                onBack={() => setStep(2)}
                onReanalyze={reanalyze}
                onFinish={() => navigate('/')}
              />
            </Step>
          )}
        </AnimatePresence>
      </main>
    </div>
  )
}

function Step({ children }: { children: React.ReactNode }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -16 }}
      transition={{ duration: 0.3 }}
      className="w-full"
    >
      {children}
    </motion.div>
  )
}

function StepHeading({
  icon: Icon,
  title,
  subtitle,
}: {
  icon: React.ComponentType<{ className?: string }>
  title: string
  subtitle: string
}) {
  return (
    <div className="flex items-start gap-3">
      <span className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-accent text-primary">
        <Icon className="h-5 w-5" />
      </span>
      <div>
        <h2 className="text-xl font-bold tracking-tight">{title}</h2>
        <p className="text-sm text-muted-foreground">{subtitle}</p>
      </div>
    </div>
  )
}

function WelcomeStep({
  name,
  onNext,
}: {
  name?: string
  onNext: () => void
}) {
  const first = name?.split(' ')[0] ?? 'à vous'
  return (
    <div className="mx-auto max-w-2xl text-center">
      <motion.div
        variants={container}
        initial="hidden"
        animate="show"
        className="flex flex-col items-center"
      >
        <motion.div
          variants={item}
          className="relative mb-6 flex h-20 w-20 items-center justify-center rounded-2xl bg-primary text-primary-foreground"
        >
          <Rocket className="h-9 w-9" />
          <span className="absolute -right-2 -top-2 flex h-7 w-7 items-center justify-center rounded-full bg-white shadow-sm ring-1 ring-border">
            <Sparkles className="h-3.5 w-3.5 text-primary" />
          </span>
        </motion.div>

        <motion.h1
          variants={item}
          className="text-3xl font-bold tracking-tight sm:text-4xl"
        >
          Bienvenue {first} 👋
        </motion.h1>
        <motion.p
          variants={item}
          className="mt-3 max-w-lg text-muted-foreground"
        >
          Workeo orchestre une équipe d’agents IA pour analyser votre CV,
          comprendre votre profil et dénicher les meilleures opportunités.
        </motion.p>

        <motion.div
          variants={item}
          className="mt-8 grid w-full gap-3 sm:grid-cols-3"
        >
          {[
            { icon: Brain, t: 'Analyse IA', d: 'CV décodé en quelques secondes' },
            { icon: Target, t: 'Matching', d: 'Score de compatibilité clair' },
            { icon: BadgeCheck, t: 'Prêt à postuler', d: 'Suivi des candidatures' },
          ].map((f) => (
            <div
              key={f.t}
              className="rounded-xl border bg-card p-4 text-left"
            >
              <f.icon className="h-5 w-5 text-primary" />
              <p className="mt-3 font-semibold">{f.t}</p>
              <p className="text-xs text-muted-foreground">{f.d}</p>
            </div>
          ))}
        </motion.div>

        <motion.div variants={item} className="mt-8">
          <Button size="lg" onClick={onNext}>
            Créer mon profil <ArrowRight className="h-4 w-4" />
          </Button>
        </motion.div>
      </motion.div>
    </div>
  )
}

function CvStep({
  file,
  setFile,
  analyzing,
  agentStep,
  onBack,
  onAnalyze,
  onSkip,
}: {
  file: File | null
  setFile: (f: File | null) => void
  analyzing: boolean
  agentStep: number
  onBack: () => void
  onAnalyze: () => void
  onSkip: () => void
}) {
  const inputRef = useRef<HTMLInputElement>(null)
  const [dragging, setDragging] = useState(false)

  return (
    <div className="space-y-6">
      <StepHeading
        icon={UploadCloud}
        title="Ajoutez votre CV"
        subtitle="PDF, DOCX ou TXT. L’IA en extrait compétences, expériences et formation."
      />

      <div
        onDragOver={(e) => {
          e.preventDefault()
          setDragging(true)
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={(e) => {
          e.preventDefault()
          setDragging(false)
          const f = e.dataTransfer.files?.[0]
          if (f) setFile(f)
        }}
        onClick={() => inputRef.current?.click()}
        className={cn(
          'flex cursor-pointer flex-col items-center justify-center rounded-2xl border-2 border-dashed px-6 py-12 text-center transition-colors',
          dragging
            ? 'border-primary bg-accent'
            : 'border-input hover:border-primary/60 hover:bg-accent/40',
        )}
      >
        <input
          ref={inputRef}
          type="file"
          accept=".pdf,.docx,.txt,.md"
          className="hidden"
          onChange={(e) => setFile(e.target.files?.[0] ?? null)}
        />
        <span className="flex h-14 w-14 items-center justify-center rounded-full bg-primary/10 text-primary">
          <UploadCloud className="h-6 w-6" />
        </span>
        {file ? (
          <>
            <p className="mt-4 font-medium">{file.name}</p>
            <p className="text-sm text-muted-foreground">
              {(file.size / 1024).toFixed(0)} Ko · cliquez pour changer de fichier
            </p>
          </>
        ) : (
          <>
            <p className="mt-4 font-medium">
              Glissez votre CV ici ou cliquez pour parcourir
            </p>
            <p className="text-sm text-muted-foreground">
              Formats acceptés : PDF, DOCX, TXT
            </p>
          </>
        )}
      </div>

      <AnimatePresence>
        {analyzing && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="overflow-hidden rounded-xl border bg-card"
          >
            <div className="space-y-1 p-4">
              {AGENT_STEPS.map((s, i) => {
                const done = i < agentStep
                const current = i === agentStep
                return (
                  <div
                    key={s.label}
                    className={cn(
                      'flex items-center gap-3 rounded-lg px-3 py-2 transition-colors',
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
                        <s.icon className="h-4 w-4" />
                      )}
                    </span>
                    <span
                      className={cn(
                        'text-sm',
                        current ? 'font-medium' : 'text-muted-foreground',
                      )}
                    >
                      {s.label}
                    </span>
                  </div>
                )
              })}
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <div className="flex items-center justify-between pt-2">
        <Button variant="ghost" onClick={onBack} disabled={analyzing}>
          <ArrowLeft className="h-4 w-4" /> Retour
        </Button>
        <div className="flex items-center gap-2">
          <Button variant="ghost" onClick={onSkip} disabled={analyzing}>
            Passer
          </Button>
          <Button onClick={onAnalyze} disabled={analyzing || !file}>
            {analyzing ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Wand2 className="h-4 w-4" />
            )}
            Analyser mon CV
          </Button>
        </div>
      </div>
    </div>
  )
}

function ReviewStep({
  profile,
  analyzing,
  agentStep,
  onBack,
  onReanalyze,
  onFinish,
}: {
  profile: Profile | null
  analyzing: boolean
  agentStep: number
  onBack: () => void
  onReanalyze: () => void
  onFinish: () => void
}) {
  const analyzed = !!profile?.category || (profile?.skills.length ?? 0) > 0

  return (
    <div className="space-y-6">
      <StepHeading
        icon={Sparkles}
        title={analyzed ? 'Votre profil est prêt' : 'Profil créé'}
        subtitle={
          analyzed
            ? 'Voici ce que l’IA a compris. Vous pourrez affiner plus tard.'
            : 'Ajoutez un CV depuis la page profil pour bénéficier de l’analyse IA.'
        }
      />

      {analyzing ? (
        <div className="rounded-xl border bg-card p-4">
          {AGENT_STEPS.map((s, i) => (
            <div
              key={s.label}
              className={cn(
                'flex items-center gap-3 rounded-lg px-3 py-2',
                i === agentStep && 'bg-accent',
              )}
            >
              <span className="flex h-7 w-7 items-center justify-center rounded-full bg-muted">
                {i < agentStep ? (
                  <CheckCircle2 className="h-4 w-4 text-emerald-600" />
                ) : i === agentStep ? (
                  <Loader2 className="h-4 w-4 animate-spin text-primary" />
                ) : (
                  <s.icon className="h-4 w-4 text-muted-foreground" />
                )}
              </span>
              <span className="text-sm">{s.label}</span>
            </div>
          ))}
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="rounded-xl border bg-card p-5 sm:col-span-2">
            <p className="text-lg font-semibold">
              {profile?.headline || profile?.name || 'Profil'}
            </p>
            <p className="text-sm text-muted-foreground">
              {profile?.full_name}
              {profile?.location ? ` · ${profile.location}` : ''}
            </p>
            {profile?.summary && (
              <p className="mt-3 text-sm text-muted-foreground">
                {profile.summary}
              </p>
            )}
          </div>

          <InfoCard
            icon={Target}
            title="Catégorie"
            empty="Non détectée"
            value={profile?.category}
          />
          <InfoCard
            icon={Briefcase}
            title="Séniorité"
            empty="Non détectée"
            value={
              profile?.seniority
                ? `${profile.seniority}${
                    profile.years_experience
                      ? ` · ${profile.years_experience} ans`
                      : ''
                  }`
                : null
            }
          />

          <TagCard
            icon={Sparkles}
            title="Compétences"
            tags={profile?.skills ?? []}
            empty="Aucune compétence détectée"
          />
          <TagCard
            icon={Target}
            title="Postes ciblés"
            tags={profile?.target_roles ?? []}
            empty="Aucun poste suggéré"
          />
          <TagCard
            icon={ScanLine}
            title="Mots-clés de recherche"
            tags={profile?.search_keywords ?? []}
            empty="Aucun mot-clé"
          />
          <TagCard
            icon={LanguagesIcon}
            title="Langues"
            tags={(profile?.languages ?? []).map((l) =>
              l.level ? `${l.name} (${l.level})` : l.name,
            )}
            empty="Aucune langue"
          />
        </div>
      )}

      <div className="flex items-center justify-between pt-2">
        <Button variant="ghost" onClick={onBack} disabled={analyzing}>
          <ArrowLeft className="h-4 w-4" /> Retour
        </Button>
        <div className="flex items-center gap-2">
          {profile?.cv_text && (
            <Button variant="outline" onClick={onReanalyze} disabled={analyzing}>
              <Wand2 className="h-4 w-4" /> Réanalyser
            </Button>
          )}
          <Button onClick={onFinish} disabled={analyzing}>
            Accéder au tableau de bord <ArrowRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  )
}

function InfoCard({
  icon: Icon,
  title,
  value,
  empty,
}: {
  icon: React.ComponentType<{ className?: string }>
  title: string
  value?: string | null
  empty: string
}) {
  return (
    <div className="rounded-xl border bg-card p-5">
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Icon className="h-4 w-4" />
        {title}
      </div>
      <p className="mt-2 font-medium">{value || empty}</p>
    </div>
  )
}

function TagCard({
  icon: Icon,
  title,
  tags,
  empty,
}: {
  icon: React.ComponentType<{ className?: string }>
  title: string
  tags: string[]
  empty: string
}) {
  return (
    <div className="rounded-xl border bg-card p-5">
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Icon className="h-4 w-4" />
        {title}
      </div>
      {tags.length > 0 ? (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {tags.slice(0, 12).map((t) => (
            <Badge key={t} variant="secondary" className="font-normal">
              {t}
            </Badge>
          ))}
        </div>
      ) : (
        <p className="mt-2 text-sm text-muted-foreground">{empty}</p>
      )}
    </div>
  )
}
