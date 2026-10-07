// This file is ONLY imported in development builds.
// vite.config.ts never references it in production; the import in main.tsx
// is wrapped in import.meta.env.DEV so it is tree-shaken from prod bundles.

import { useAuth } from './AuthContext'

const DEV_TOKEN = 'dev-bypass-token'
const DEV_USER_ID = '00000000-0000-0000-0000-000000000001'

export function DevLoginBanner() {
  const { token, signIn, signOut } = useAuth()

  return (
    <div className="fixed bottom-0 left-0 right-0 z-50 bg-amber-900/80 border-t border-amber-500/40 px-4 py-2 flex items-center gap-4 text-sm font-mono backdrop-blur">
      <span className="text-amber-400 font-semibold">DEV</span>
      <span className="text-amber-200/70">auth bypass active</span>
      <div className="ml-auto flex gap-2">
        {token ? (
          <button
            onClick={signOut}
            className="px-3 py-1 rounded border border-amber-500/50 text-amber-300 hover:bg-amber-500/20 transition-colors"
          >
            Sign out
          </button>
        ) : (
          <button
            onClick={() => signIn(DEV_TOKEN, DEV_USER_ID)}
            className="px-3 py-1 rounded border border-amber-500/50 text-amber-300 hover:bg-amber-500/20 transition-colors"
          >
            Sign in as dev user
          </button>
        )}
      </div>
    </div>
  )
}
