import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'

import App from './App'
import './index.css'
import { AuthProvider } from './contexts/auth'
import { ProfilesProvider } from './contexts/profiles'
import { Toaster } from './components/ui/sonner'
import { TooltipProvider } from './components/ui/tooltip'
import { applyTheme, getStoredTheme } from './lib/theme'

// Applique le thème mémorisé avant le premier rendu (évite un flash).
applyTheme(getStoredTheme())

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <TooltipProvider delayDuration={200}>
        <AuthProvider>
          <ProfilesProvider>
            <App />
            <Toaster />
          </ProfilesProvider>
        </AuthProvider>
      </TooltipProvider>
    </BrowserRouter>
  </StrictMode>,
)
