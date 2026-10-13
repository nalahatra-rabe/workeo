import {
  BadgeCheck,
  Bookmark,
  Building2,
  ExternalLink,
  MapPin,
  Wifi,
} from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import type { Job } from '@/lib/types'
import { cn } from '@/lib/utils'

function scoreStyle(score: number) {
  if (score >= 75) return 'bg-emerald-100 text-emerald-700'
  if (score >= 50) return 'bg-blue-100 text-blue-700'
  if (score >= 25) return 'bg-amber-100 text-amber-700'
  return 'bg-slate-100 text-slate-600'
}

export function JobCard({
  job,
  score,
  reasons,
  saved,
  onSave,
  onOpen,
}: {
  job: Job
  score?: number
  reasons?: string[]
  saved?: boolean
  onSave?: () => void
  onOpen?: () => void
}) {
  return (
    <div className="group flex flex-col rounded-xl border bg-card p-5 transition-colors hover:border-primary/40">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <button
            onClick={onOpen}
            className="text-left text-base font-semibold leading-snug hover:text-primary"
          >
            {job.title}
          </button>
          <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-muted-foreground">
            <span className="inline-flex items-center gap-1.5">
              <Building2 className="h-3.5 w-3.5" />
              {job.company}
            </span>
            {job.location && (
              <span className="inline-flex items-center gap-1.5">
                <MapPin className="h-3.5 w-3.5" />
                {job.location}
              </span>
            )}
            {job.remote && (
              <span className="inline-flex items-center gap-1.5 text-primary">
                <Wifi className="h-3.5 w-4" /> Télétravail
              </span>
            )}
          </div>
        </div>

        {typeof score === 'number' && (
          <span
            className={cn(
              'flex shrink-0 flex-col items-center rounded-lg px-2.5 py-1.5 text-center',
              scoreStyle(score),
            )}
          >
            <span className="text-sm font-bold leading-none">{score}</span>
            <span className="text-[10px] font-medium uppercase opacity-70">
              match
            </span>
          </span>
        )}
      </div>

      {(job.summary || job.description) && (
        <p className="mt-3 line-clamp-2 text-sm text-muted-foreground">
          {job.summary || job.description}
        </p>
      )}

      <div className="mt-3 flex flex-wrap items-center gap-1.5">
        {job.contract_type && (
          <Badge variant="accent" className="font-normal">
            {job.contract_type}
          </Badge>
        )}
        {job.category && (
          <Badge variant="secondary" className="font-normal">
            {job.category}
          </Badge>
        )}
        {job.seniority && (
          <Badge variant="outline" className="font-normal">
            {job.seniority}
          </Badge>
        )}
      </div>

      {reasons && reasons.length > 0 && (
        <ul className="mt-3 space-y-1">
          {reasons.slice(0, 3).map((r) => (
            <li
              key={r}
              className="flex items-start gap-1.5 text-xs text-muted-foreground"
            >
              <BadgeCheck className="mt-0.5 h-3.5 w-3.5 shrink-0 text-primary" />
              {r}
            </li>
          ))}
        </ul>
      )}

      <div className="mt-4 flex items-center justify-between border-t pt-4">
        <div className="flex items-center gap-2">
          <span className="inline-flex h-6 items-center gap-1 rounded-md bg-muted px-2 text-xs text-muted-foreground">
            <Building2 className="h-3 w-3" /> {job.source}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant={saved ? 'secondary' : 'outline'}
            size="sm"
            onClick={onSave}
            disabled={saved}
            className={cn(saved && 'text-emerald-600')}
          >
            <Bookmark className={cn('h-4 w-4', saved && 'fill-current')} />
            {saved ? 'Suivie' : 'Suivre'}
          </Button>
          {job.url && (
            <Button variant="ghost" size="sm" asChild>
              <a href={job.url} target="_blank" rel="noreferrer">
                <ExternalLink className="h-4 w-4" /> Voir
              </a>
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}
