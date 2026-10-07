import type { ReactNode } from 'react'
import { Link, useLocation } from 'react-router-dom'

interface LayoutProps {
  children: ReactNode
}

export function Layout({ children }: LayoutProps) {
  const location = useLocation()
  const inStory = location.pathname.startsWith('/stories')

  return (
    <div className="min-h-screen flex flex-col">
      <header className="border-b border-g-border bg-g-surface/80 backdrop-blur sticky top-0 z-40">
        <div className="max-w-4xl mx-auto px-6 h-14 flex items-center gap-6">
          <Link
            to="/"
            className="font-serif text-lg font-medium text-g-text tracking-wide hover:text-g-accent transition-colors"
          >
            Grimoire
          </Link>
          {inStory && (
            <nav className="flex items-center gap-1 text-sm text-g-muted">
              <span>/</span>
              <Link to="/stories" className="hover:text-g-text transition-colors px-1">
                Stories
              </Link>
            </nav>
          )}
        </div>
      </header>

      <main className="flex-1 max-w-4xl mx-auto w-full px-6 py-10">
        {children}
      </main>
    </div>
  )
}
