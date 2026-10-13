import { Bookmark, ExternalLink, MapPin, Wifi } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Separator } from '@/components/ui/separator'
import type { Job } from '@/lib/types'
import { cn } from '@/lib/utils'

export function JobDialog({
  open,
  onOpenChange,
  job,
  score,
  reasons,
  saved,
  onSave,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  job: Job | null
  score?: number
  reasons?: string[]
  saved?: boolean
  onSave?: () => void
}) {
  if (!job) return null
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[85vh] max-w-2xl overflow-y-auto scrollbar-thin">
        <DialogHeader>
          <div className="flex items-start justify-between gap-4 pr-6">
            <div>
              <DialogTitle className="text-xl leading-snug">
                {job.title}
              </DialogTitle>
              <DialogDescription className="mt-1">
                {job.company}
                {job.location ? ` · ${job.location}` : ''}
              </DialogDescription>
            </div>
            {typeof score === 'number' && score > 0 && (
              <span
                className={cn(
                  'shrink-0 rounded-lg px-3 py-1.5 text-sm font-semibold',
                  score >= 75
                    ? 'bg-emerald-100 text-emerald-700'
                    : score >= 50
                      ? 'bg-blue-100 text-blue-700'
                      : 'bg-amber-100 text-amber-700',
                )}
              >
                {score}% match
              </span>
            )}
          </div>
        </DialogHeader>

        <div className="flex flex-wrap items-center gap-1.5">
          {job.remote && (
            <Badge variant="success" className="gap-1 font-normal">
              <Wifi className="h-3 w-3" /> Télétravail
            </Badge>
          )}
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
          {job.location && (
            <Badge variant="outline" className="gap-1 font-normal">
              <MapPin className="h-3 w-3" /> {job.location}
            </Badge>
          )}
        </div>

        {reasons && reasons.length > 0 && (
          <div className="rounded-xl bg-accent/60 p-4">
            <p className="text-sm font-semibold text-accent-foreground">
              Pourquoi cette offre vous correspond
            </p>
            <ul className="mt-2 space-y-1">
              {reasons.map((r) => (
                <li
                  key={r}
                  className="flex items-start gap-2 text-sm text-accent-foreground/90"
                >
                  <span className="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-primary" />
                  {r}
                </li>
              ))}
            </ul>
          </div>
        )}

        <Separator />

        <div className="whitespace-pre-line text-sm leading-relaxed text-foreground/90">
          {job.description || job.summary || 'Aucune description fournie.'}
        </div>

        {job.skills.length > 0 && (
          <div className="flex flex-wrap gap-1.5">
            {job.skills.map((s) => (
              <Badge key={s} variant="secondary" className="font-normal">
                {s}
              </Badge>
            ))}
          </div>
        )}

        <DialogFooter className="gap-2">
          {job.skills.length > 0 && <span className="hidden" />}
          <Button
            variant={saved ? 'secondary' : 'outline'}
            onClick={onSave}
            disabled={saved}
          >
            <Bookmark className={cn('h-4 w-4', saved && 'fill-current')} />
            {saved ? 'Déjà suivie' : 'Suivre cette offre'}
          </Button>
          {job.url && (
            <Button asChild>
              <a href={job.url} target="_blank" rel="noreferrer">
                <ExternalLink className="h-4 w-4" /> Postuler
              </a>
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
