import { BriefcaseBusiness } from 'lucide-react'
import { Link } from 'react-router-dom'

import { cn } from '@/lib/utils'

export function Logo({
  className,
  showText = true,
  to = '/',
}: {
  className?: string
  showText?: boolean
  to?: string
}) {
  return (
    <Link
      to={to}
      className={cn('inline-flex items-center gap-2 select-none', className)}
    >
      <span className="flex h-9 w-9 items-center justify-center rounded-lg bg-primary text-primary-foreground">
        <BriefcaseBusiness className="h-5 w-5" />
      </span>
      {showText && (
        <span className="text-xl font-bold tracking-tight text-foreground">
          Workeo
        </span>
      )}
    </Link>
  )
}
