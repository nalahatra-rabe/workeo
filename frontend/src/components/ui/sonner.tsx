import { Toaster as Sonner, type ToasterProps } from 'sonner'

/** Toaster Sonner thémé pour Workeo. */
export function Toaster(props: ToasterProps) {
  return (
    <Sonner
      position="top-right"
      richColors
      closeButton
      toastOptions={{
        classNames: {
          toast:
            'rounded-xl border bg-background text-foreground shadow-lg',
        },
      }}
      {...props}
    />
  )
}
