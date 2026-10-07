import { createContext, useContext, useState, useCallback, type ReactNode } from 'react'
import { setToken, clearToken } from '@/api/client'

interface AuthState {
  token: string | null
  userId: string | null
  signIn: (token: string, userId: string) => void
  signOut: () => void
}

const AuthContext = createContext<AuthState | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setTokenState] = useState<string | null>(null)
  const [userId, setUserId] = useState<string | null>(null)

  const signIn = useCallback((t: string, uid: string) => {
    setToken(t)
    setTokenState(t)
    setUserId(uid)
  }, [])

  const signOut = useCallback(() => {
    clearToken()
    setTokenState(null)
    setUserId(null)
  }, [])

  return (
    <AuthContext.Provider value={{ token, userId, signIn, signOut }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used inside AuthProvider')
  return ctx
}
