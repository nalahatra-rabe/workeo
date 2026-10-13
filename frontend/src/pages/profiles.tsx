import {
  CheckCircle2,
  FileText,
  Loader2,
  MapPin,
  Plus,
  Star,
  Trash2,
} from 'lucide-react'
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
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
import { cn } from '@/lib/utils'

const CONTRACTS = ['Alternance', 'Stage', 'CDI', 'CDD', 'Freelance']

export function ProfileListPage() {
  const { profiles, setActive, setDefault, createProfile, deleteProfile } =
    useProfiles()
  const navigate = useNavigate()

  const [open, setOpen] = useState(false)
  const [creating, setCreating] = useState(false)
  const [name, setName] = useState('')
  const [fullName, setFullName] = useState('')
  const [headline, setHeadline] = useState('')
  const [location, setLocation] = useState('')
  const [remote, setRemote] = useState('any')
  const [contracts, setContracts] = useState<string[]>(['Alternance'])

  function resetForm() {
    setName('')
    setFullName('')
    setHeadline('')
    setLocation('')
    setRemote('any')
    setContracts(['Alternance'])
  }

  function toggleContract(c: string) {
    setContracts((prev) =>
      prev.includes(c) ? prev.filter((x) => x !== c) : [...prev, c],
    )
  }

  async function create() {
    if (!fullName.trim()) {
      toast.error('Le nom complet est requis.')
      return
    }
    setCreating(true)
    try {
      const p = await createProfile({
        name: name.trim() || headline.trim() || 'Nouveau profil',
        full_name: fullName.trim(),
        headline: headline.trim() || undefined,
        location: location.trim() || undefined,
        remote_preference: remote,
        contract_types: contracts,
      })
      setOpen(false)
      resetForm()
      toast.success('Profil créé', { description: p.name })
      navigate('/profile')
    } catch (err) {
      toast.error('Création impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    } finally {
      setCreating(false)
    }
  }

  async function remove(id: string) {
    try {
      await deleteProfile(id)
      toast.success('Profil supprimé')
    } catch (err) {
      toast.error('Suppression impossible', {
        description: err instanceof Error ? err.message : undefined,
      })
    }
  }

  return (
    <div className="mx-auto max-w-5xl space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">Vos profils</h2>
          <p className="text-sm text-muted-foreground">
            Chaque profil possède ses propres données, recherches et
            candidatures.
          </p>
        </div>
        <Button
          onClick={() => {
            resetForm()
            setOpen(true)
          }}
        >
          <Plus className="h-4 w-4" /> Nouveau profil
        </Button>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        {profiles.map((p) => (
          <Card key={p.id}>
            <CardContent className="p-5">
              <div className="flex items-center gap-3">
                <span className="flex h-11 w-11 items-center justify-center rounded-xl bg-primary/10 text-lg font-bold text-primary">
                  {p.name.slice(0, 1).toUpperCase()}
                </span>
                <div>
                  <div className="flex items-center gap-2">
                    <p className="font-semibold">{p.name}</p>
                    {p.is_default && (
                      <Badge variant="accent" className="gap-1 font-normal">
                        <Star className="h-3 w-3" /> défaut
                      </Badge>
                    )}
                  </div>
                  <p className="text-sm text-muted-foreground">
                    {p.headline || p.full_name}
                  </p>
                </div>
              </div>

              <div className="mt-4 flex flex-wrap gap-1.5">
                {p.category && (
                  <Badge variant="secondary" className="font-normal">
                    {p.category}
                  </Badge>
                )}
                {p.location && (
                  <Badge variant="outline" className="gap-1 font-normal">
                    <MapPin className="h-3 w-3" /> {p.location}
                  </Badge>
                )}
                {p.cv_filename && (
                  <Badge variant="outline" className="gap-1 font-normal">
                    <FileText className="h-3 w-3" /> CV
                  </Badge>
                )}
              </div>

              {p.skills.length > 0 && (
                <p className="mt-3 line-clamp-1 text-xs text-muted-foreground">
                  {p.skills.slice(0, 6).join(' · ')}
                </p>
              )}

              <div className="mt-5 flex items-center gap-2 border-t pt-4">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    setActive(p.id)
                    toast.success(`Profil « ${p.name} » activé`)
                  }}
                >
                  Sélectionner
                </Button>
                {!p.is_default && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setDefault(p.id)}
                  >
                    <CheckCircle2 className="h-4 w-4" /> Par défaut
                  </Button>
                )}
                <Button
                  variant="ghost"
                  size="sm"
                  className="ml-auto text-destructive hover:text-destructive"
                  onClick={() => remove(p.id)}
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Nouveau profil</DialogTitle>
            <DialogDescription>
              Créez un profil distinct pour cibler d’autres opportunités.
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="p-name">Nom du profil</Label>
              <Input
                id="p-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Ex : Alternance Dev Web"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="p-fullname">Nom complet *</Label>
              <Input
                id="p-fullname"
                value={fullName}
                onChange={(e) => setFullName(e.target.value)}
              />
            </div>
            <div className="space-y-2 sm:col-span-2">
              <Label htmlFor="p-headline">Titre / objectif</Label>
              <Input
                id="p-headline"
                value={headline}
                onChange={(e) => setHeadline(e.target.value)}
                placeholder="Ex : Développeur full-stack en alternance"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="p-location">Localisation</Label>
              <Input
                id="p-location"
                value={location}
                onChange={(e) => setLocation(e.target.value)}
              />
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

          <div className="flex justify-end gap-2 pt-2">
            <Button variant="ghost" onClick={() => setOpen(false)}>
              Annuler
            </Button>
            <Button onClick={create} disabled={creating}>
              {creating ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Plus className="h-4 w-4" />
              )}
              Créer
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
