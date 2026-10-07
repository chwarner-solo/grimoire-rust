import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { AuthProvider } from '@/auth/AuthContext'
import { App } from './App'
import './index.css'

const root = document.getElementById('root')!

// The DevLoginBanner import and render are wrapped in import.meta.env.DEV.
// Vite replaces import.meta.env.DEV with `false` in production builds and
// tree-shakes the entire branch — DevLogin.tsx is never included in prod.
async function mount() {
  if (import.meta.env.DEV) {
    const { DevLoginBanner } = await import('./auth/DevLogin')
    createRoot(root).render(
      <StrictMode>
        <BrowserRouter>
          <AuthProvider>
            <App />
            <DevLoginBanner />
          </AuthProvider>
        </BrowserRouter>
      </StrictMode>,
    )
  } else {
    createRoot(root).render(
      <StrictMode>
        <BrowserRouter>
          <AuthProvider>
            <App />
          </AuthProvider>
        </BrowserRouter>
      </StrictMode>,
    )
  }
}

mount()
