import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

/** Fusionne des classes Tailwind en gérant les conflits. */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** Formate une date ISO en français court. */
export function formatDate(input?: string | null): string {
  if (!input) return '—'
  const d = new Date(input)
  if (Number.isNaN(d.getTime())) return '—'
  return new Intl.DateTimeFormat('fr-FR', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  }).format(d)
}

export function relativeTime(iso?: string | null): string {
  if (!iso) return '—'
  const date = new Date(iso)
  const diff = Date.now() - date.getTime()
  const mins = Math.round(diff / 60000)
  if (Math.abs(mins) < 1) return "à l'instant"
  if (Math.abs(mins) < 60) return `il y a ${mins} min`
  const hours = Math.round(mins / 60)
  if (hours < 24) return `il y a ${hours} h`
  const days = Math.round(hours / 24)
  if (days < 30) return `il y a ${days} j`
  return new Date(iso).toLocaleDateString('fr-FR')
}

/** Initiales à partir d'un nom. */
export function initials(name: string): string {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? '')
    .join('')
}

export function initialsColor(seed: string): string {
  const palette = [
    'bg-blue-100 text-blue-700',
    'bg-sky-100 text-sky-700',
    'bg-indigo-100 text-indigo-700',
    'bg-cyan-100 text-cyan-700',
    'bg-blue-200 text-blue-800',
  ]
  let h = 0
  for (const ch of seed) h = (h * 31 + ch.charCodeAt(0)) % 997
  return palette[h % palette.length]
}
