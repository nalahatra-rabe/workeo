import {
  Award,
  Briefcase,
  CheckCircle2,
  FileText,
  GraduationCap,
  Languages as LanguagesIcon,
  Loader2,
  MapPin,
  RefreshCw,
  Sparkles,
  Target,
  UploadCloud,
} from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useProfiles } from '@/contexts/profiles'
import { cn } from '@/lib/utils'

const CONTRACTS = ['Alternance', 'Stage', 'CDI', 'CDD', 'Freelance']

export function ProfilePage() {
  const { activeProfile, updateProfile, uploadCv, analyze, setDefault } =
    useProfiles()
  const fileRef = useRef<HTMLInputElement>(null)

  const [headline, setHeadline] = useState('')
  const [location, setLocation] = useState('')
  const [summary, setSummary] = useState('')
  const [remote, setRemote] = useState('any')
  const [contracts, setContracts] = useState<string[]>([])
  const [salary, setSalary] = useState('')
  const [saving, setSaving] = useState(false)
  const [uploading, setUploading] = useState(false)
  const [analyzing, setAnalyzing] = useState(false)

  useEffect(() => {
    if (!activeProfile) return
    setHeadline(activeProfile.headline ?? '')
    setLocation(activeProfile.location ?? '')
    setSummary(activeProfile.summary ?? '')
    setRemote(activeProfile.remote_preference ?? 'any')
    setContracts(activeProfile.contract_types ?? [])
    setSalary(activeProfile.salary_min ? String(activeProfile.salary_min) : '')
  }, [activeProfile])

  if (!activeProfile) {
    return (
      <div className="flex min-h-[50vh] items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  function toggleContract(c: string) {
    setContracts((prev) =>
      prev.includes(c) ? prev.filter((x) => x !== c) : [...prev, c],
    )
  }

  async function save() {
    if (!activeProfile) return
    setSaving(true)
    try {
      await updateProfile(activeProfile.id, {
        headline,
        location,
        summary,
        remote_preference: remote,
        contract_types: contracts,
        salary_min: salary ? Number(salary) : undefined,
      })
      toast.success('Profil mis à jour')
    } catch (err) {
      toast.error('Enregistrement impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    } finally {
      setSaving(false)
    }
  }

  async function onUpload(file: File) {
    if (!activeProfile) return
    setUploading(true)
    try {
      const p = await uploadCv(activeProfile.id, file)
      toast.success('CV importé et analysé', {
        description: `${p.skills.length} compétences détectées.`,
      })
    } catch (err) {
      toast.error('Import impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    } finally {
      setUploading(false)
    }
  }

  async function reanalyze() {
    if (!activeProfile) return
    setAnalyzing(true)
    try {
      await analyze(activeProfile.id)
      toast.success('Analyse terminée')
    } catch (err) {
      toast.error('Analyse impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    } finally {
      setAnalyzing(false)
    }
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      {/* En-tête */}
      <div className="flex flex-col gap-4 rounded-2xl border bg-card p-6 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-4">
          <span className="flex h-14 w-14 items-center justify-center rounded-2xl bg-primary/10 text-2xl font-bold text-primary">
            {activeProfile.name.slice(0, 1).toUpperCase()}
          </span>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-xl font-bold">{activeProfile.name}</h2>
              {activeProfile.is_default && (
                <Badge variant="accent" className="gap-1 font-normal">
                  <CheckCircle2 className="h-3 w-3" /> par défaut
                </Badge>
              )}
            </div>
            <p className="text-sm text-muted-foreground">
              {activeProfile.headline || activeProfile.full_name}
            </p>
          </div>
        </div>
        {!activeProfile.is_default && (
          <Button variant="outline" onClick={() => setDefault(activeProfile.id)}>
            Définir par défaut
          </Button>
        )}
      </div>

      <Tabs defaultValue="info">
        <TabsList>
          <TabsTrigger value="info">
            <Briefcase className="h-4 w-4" /> Informations
          </TabsTrigger>
          <TabsTrigger value="cv">
            <FileText className="h-4 w-4" /> CV & analyse
          </TabsTrigger>
        </TabsList>

        <TabsContent value="info">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Préférences de recherche</CardTitle>
              <CardDescription>
                Ces informations personnalisent les recherches et le matching.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-5">
              <div className="grid gap-5 sm:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="p-headline">Titre / objectif</Label>
                  <Input
                    id="p-headline"
                    value={headline}
                    onChange={(e) => setHeadline(e.target.value)}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="p-location">Localisation</Label>
                  <div className="relative">
                    <MapPin className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input
                      id="p-location"
                      className="pl-9"
                      value={location}
                      onChange={(e) => setLocation(e.target.value)}
                    />
                  </div>
                </div>
                <div className="space-y-2">
                  <Label>Télétravail</Label>
                  <Select value={remote} onValueChange={setRemote}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="any">Peu importe</SelectItem>
                      <SelectItem value="remote">100 % télétravail</SelectItem>
                      <SelectItem value="hybrid">Hybride</SelectItem>
                      <SelectItem value="on-site">Sur site</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="p-salary">Salaire minimum (€ / an)</Label>
                  <Input
                    id="p-salary"
                    type="number"
                    value={salary}
                    onChange={(e) => setSalary(e.target.value)}
                    placeholder="Ex : 24000"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label>Types de contrat</Label>
                <div className="flex flex-wrap gap-2">
                  {CONTRACTS.map((c) => (
                    <button
                      key={c}
                      type="button"
                      onClick={() => toggleContract(c)}
                      className={cn(
                        'rounded-full border px-3.5 py-1.5 text-sm font-medium transition-colors',
                        contracts.includes(c)
                          ? 'border-primary bg-primary text-primary-foreground'
                          : 'border-input hover:bg-accent',
                      )}
                    >
                      {c}
                    </button>
                  ))}
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="p-summary">Résumé</Label>
                <textarea
                  id="p-summary"
                  rows={4}
                  value={summary}
                  onChange={(e) => setSummary(e.target.value)}
                  className="flex w-full rounded-lg border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  placeholder="Quelques lignes sur votre parcours et vos objectifs…"
                />
              </div>

              <div className="flex justify-end">
                <Button onClick={save} disabled={saving}>
                  {saving && <Loader2 className="h-4 w-4 animate-spin" />}
                  Enregistrer
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="cv">
          <div className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Curriculum vitæ</CardTitle>
                <CardDescription>
                  Importez un PDF, DOCX ou TXT : l’agent IA en extrait les
                  informations clés.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex flex-wrap items-center gap-3">
                  <input
                    ref={fileRef}
                    type="file"
                    accept=".pdf,.docx,.txt,.md"
                    className="hidden"
                    onChange={(e) => {
                      const f = e.target.files?.[0]
                      if (f) onUpload(f)
                      e.target.value = ''
                    }}
                  />
                  <Button
                    onClick={() => fileRef.current?.click()}
                    disabled={uploading}
                  >
                    {uploading ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <UploadCloud className="h-4 w-4" />
                    )}
                    {activeProfile.cv_filename ? 'Remplacer le CV' : 'Importer un CV'}
                  </Button>
                  {activeProfile.cv_text && (
                    <Button
                      variant="outline"
                      onClick={reanalyze}
                      disabled={analyzing}
                    >
                      {analyzing ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      ) : (
                        <RefreshCw className="h-4 w-4" />
                      )}
                      Réanalyser
                    </Button>
                  )}
                </div>

                {activeProfile.cv_filename && (
                  <div className="flex items-center gap-2 rounded-lg bg-muted px-3 py-2 text-sm">
                    <FileText className="h-4 w-4 text-muted-foreground" />
                    {activeProfile.cv_filename}
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <div className="flex items-center gap-2">
                  <span className="flex h-9 w-9 items-center justify-center rounded-lg bg-accent text-primary">
                    <Sparkles className="h-4 w-4" />
                  </span>
                  <div>
                    <CardTitle className="text-base">
                      Analyse de l’agent IA
                    </CardTitle>
                    <CardDescription>
                      Éléments extraits et inférés depuis votre CV.
                    </CardDescription>
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-5">
                <div className="grid gap-4 sm:grid-cols-2">
                  <Field
                    icon={Target}
                    label="Catégorie"
                    value={activeProfile.category}
                  />
                  <Field
                    icon={Award}
                    label="Séniorité"
                    value={
                      activeProfile.seniority
                        ? `${activeProfile.seniority}${
                            activeProfile.years_experience
                              ? ` · ${activeProfile.years_experience} ans`
                              : ''
                          }`
                        : null
                    }
                  />
                </div>

                <Separator />

                <TagSection
                  icon={Sparkles}
                  title="Compétences"
                  tags={activeProfile.skills}
                />
                <TagSection
                  icon={Target}
                  title="Postes ciblés"
                  tags={activeProfile.target_roles}
                />
                <TagSection
                  icon={Sparkles}
                  title="Mots-clés de recherche"
                  tags={activeProfile.search_keywords}
                />
                <TagSection
                  icon={LanguagesIcon}
                  title="Langues"
                  tags={activeProfile.languages.map((l) =>
                    l.level ? `${l.name} (${l.level})` : l.name,
                  )}
                />

                {activeProfile.experiences.length > 0 && (
                  <div>
                    <p className="mb-2 flex items-center gap-2 text-sm font-medium text-muted-foreground">
                      <Briefcase className="h-4 w-4" /> Expériences
                    </p>
                    <div className="space-y-2">
                      {activeProfile.experiences.map((e, i) => (
                        <div key={i} className="rounded-lg border p-3">
                          <p className="text-sm font-medium">
                            {e.title || 'Poste'}
                            {e.company ? ` · ${e.company}` : ''}
                          </p>
                          {(e.start || e.end) && (
                            <p className="text-xs text-muted-foreground">
                              {e.start} {e.end ? `– ${e.end}` : ''}
                            </p>
                          )}
                          {e.description && (
                            <p className="mt-1 text-sm text-muted-foreground">
                              {e.description}
                            </p>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {activeProfile.education.length > 0 && (
                  <div>
                    <p className="mb-2 flex items-center gap-2 text-sm font-medium text-muted-foreground">
                      <GraduationCap className="h-4 w-4" /> Formation
                    </p>
                    <div className="space-y-2">
                      {activeProfile.education.map((e, i) => (
                        <div key={i} className="rounded-lg border p-3">
                          <p className="text-sm font-medium">
                            {e.degree || 'Diplôme'}
                          </p>
                          <p className="text-xs text-muted-foreground">
                            {[e.school, e.year].filter(Boolean).join(' · ')}
                          </p>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {!activeProfile.cv_text && (
                  <div className="flex flex-col items-center rounded-xl border border-dashed py-10 text-center">
                    <UploadCloud className="h-6 w-6 text-muted-foreground" />
                    <p className="mt-3 font-medium">Aucun CV analysé</p>
                    <p className="mt-1 max-w-sm text-sm text-muted-foreground">
                      Importez un CV pour déclencher l’analyse automatique.
                    </p>
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}

function Field({
  icon: Icon,
  label,
  value,
}: {
  icon: React.ComponentType<{ className?: string }>
  label: string
  value?: string | null
}) {
  return (
    <div className="rounded-xl border p-4">
      <p className="flex items-center gap-2 text-sm text-muted-foreground">
        <Icon className="h-4 w-4" /> {label}
      </p>
      <p className="mt-1.5 font-medium">{value || 'Non détecté'}</p>
    </div>
  )
}

function TagSection({
  icon: Icon,
  title,
  tags,
}: {
  icon: React.ComponentType<{ className?: string }>
  title: string
  tags: string[]
}) {
  if (tags.length === 0) return null
  return (
    <div>
      <p className="mb-2 flex items-center gap-2 text-sm font-medium text-muted-foreground">
        <Icon className="h-4 w-4" /> {title}
      </p>
      <div className="flex flex-wrap gap-1.5">
        {tags.map((t) => (
          <Badge key={t} variant="secondary" className="font-normal">
            {t}
          </Badge>
        ))}
      </div>
    </div>
  )
}
