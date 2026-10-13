import { Check, ChevronDown, Settings2 } from 'lucide-react'
import { useNavigate } from 'react-router-dom'

import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useProfiles } from '@/contexts/profiles'

export function ProfileSwitcher() {
  const { profiles, activeProfile, setActive } = useProfiles()
  const navigate = useNavigate()

  if (!activeProfile) return null

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          className="h-10 max-w-[230px] justify-between gap-2"
        >
          <span className="flex items-center gap-2 truncate">
            <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-primary/10 text-xs font-bold text-primary">
              {activeProfile.name.slice(0, 1).toUpperCase()}
            </span>
            <span className="truncate font-medium">{activeProfile.name}</span>
          </span>
          <ChevronDown className="h-4 w-4 shrink-0 opacity-60" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-64">
        <DropdownMenuLabel>Vos profils</DropdownMenuLabel>
        {profiles.map((p) => (
          <DropdownMenuItem
            key={p.id}
            onClick={() => setActive(p.id)}
            className="justify-between"
          >
            <span className="flex flex-col">
              <span className="font-medium">{p.name}</span>
              <span className="text-xs text-muted-foreground">
                {p.headline || p.full_name}
              </span>
            </span>
            <span className="flex items-center gap-1.5">
              {p.id === activeProfile.id && (
                <Check className="h-4 w-4 text-primary" />
              )}
            </span>
          </DropdownMenuItem>
        ))}
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => navigate('/profiles')}>
          <Settings2 className="h-4 w-4" /> Gérer mes profils
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
