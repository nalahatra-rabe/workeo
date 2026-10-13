export type Theme = 'light' | 'dark'

const KEY = 'workeo_theme'

export function getStoredTheme(): Theme {
  const value = localStorage.getItem(KEY)
  if (value === 'light' || value === 'dark') return value
  return window.matchMedia('(prefers-color-scheme: dark)').matches
    ? 'dark'
    : 'light'
}

export function applyTheme(theme: Theme) {
  document.documentElement.classList.toggle('dark', theme === 'dark')
}

export function storeTheme(theme: Theme) {
  localStorage.setItem(KEY, theme)
}
