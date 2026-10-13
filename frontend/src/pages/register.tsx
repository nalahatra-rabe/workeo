import { ArrowRight, Check, Eye, EyeOff, Loader2, X } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { toast } from 'sonner'

import { AuthShell } from '@/components/layout/auth-shell'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useAuth } from '@/contexts/auth'
import { cn } from '@/lib/utils'

function scorePassword(pw: string) {
  let score = 0
  if (pw.length >= 6) score++
  if (pw.length >= 10) score++
  if (/[A-Z]/.test(pw) && /[a-z]/.test(pw)) score++
  if (/\d/.test(pw)) score++
  if (/[^A-Za-z0-9]/.test(pw)) score++
  return Math.min(score, 4)
}

const LABELS = ['Très faible', 'Faible', 'Correct', 'Bon', 'Excellent']
const COLORS = [
  'bg-red-400',
  'bg-orange-400',
  'bg-amber-400',
  'bg-blue-500',
  'bg-emerald-500',
]

export function RegisterPage() {
  const { register } = useAuth()
  const navigate = useNavigate()
  const [fullName, setFullName] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [show, setShow] = useState(false)
  const [loading, setLoading] = useState(false)

  const score = useMemo(() => scorePassword(password), [password])
  const rules = [
    { ok: password.length >= 6, label: 'Au moins 6 caractères' },
    { ok: /[A-Z]/.test(password) && /[a-z]/.test(password), label: 'Maj. et min.' },
    { ok: /\d/.test(password), label: 'Un chiffre' },
  ]

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (password.length < 6) {
      toast.error('Mot de passe trop court', {
        description: 'Utilisez au moins 6 caractères.',
      })
      return
    }
    setLoading(true)
    try {
      await register(fullName, email, password)
      toast.success('Compte créé 🎉', {
        description: 'Configurons votre premier profil.',
      })
      navigate('/onboarding')
    } catch (err) {
      toast.error('Création impossible', {
        description: err instanceof Error ? err.message : 'Réessayez.',
      })
    } finally {
      setLoading(false)
    }
  }

  return (
    <AuthShell
      title="Créer votre compte"
      subtitle="Quelques secondes suffisent pour lancer votre recherche."
      footer={
        <p className="text-muted-foreground">
          Déjà inscrit ?{' '}
          <Link to="/login" className="font-medium text-primary hover:underline">
            Se connecter
          </Link>
        </p>
      }
    >
      <form onSubmit={onSubmit} className="space-y-5">
        <div className="space-y-2">
          <Label htmlFor="fullName">Nom complet</Label>
          <Input
            id="fullName"
            placeholder="Alice Martin"
            value={fullName}
            onChange={(e) => setFullName(e.target.value)}
            autoComplete="name"
            required
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="email">Adresse email</Label>
          <Input
            id="email"
            type="email"
            placeholder="vous@exemple.com"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            autoComplete="email"
            required
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="password">Mot de passe</Label>
          <div className="relative">
            <Input
              id="password"
              type={show ? 'text' : 'password'}
              placeholder="••••••••"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="new-password"
              required
              className="pr-10"
            />
            <button
              type="button"
              onClick={() => setShow((s) => !s)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              aria-label={show ? 'Masquer' : 'Afficher'}
            >
              {show ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </button>
          </div>

          {password.length > 0 && (
            <div className="space-y-2 pt-1">
              <div className="flex gap-1.5">
                {[0, 1, 2, 3].map((i) => (
                  <div
                    key={i}
                    className={cn(
                      'h-1.5 flex-1 rounded-full transition-colors',
                      i < score ? COLORS[score] : 'bg-muted',
                    )}
                  />
                ))}
              </div>
              <p className="text-xs text-muted-foreground">
                Sécurité : <span className="font-medium">{LABELS[score]}</span>
              </p>
              <ul className="flex flex-wrap gap-x-4 gap-y-1">
                {rules.map((r) => (
                  <li
                    key={r.label}
                    className={cn(
                      'inline-flex items-center gap-1 text-xs',
                      r.ok ? 'text-emerald-600' : 'text-muted-foreground',
                    )}
                  >
                    {r.ok ? (
                      <Check className="h-3 w-3" />
                    ) : (
                      <X className="h-3 w-3" />
                    )}
                    {r.label}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>

        <Button type="submit" className="w-full" disabled={loading}>
          {loading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <ArrowRight className="h-4 w-4" />
          )}
          Créer mon compte
        </Button>
      </form>
    </AuthShell>
  )
}
