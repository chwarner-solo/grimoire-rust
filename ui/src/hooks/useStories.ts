import { useState, useEffect, useCallback } from 'react'
import { listStories } from '@/api/stories'
import type { Story } from '@/types'

interface UseStoriesResult {
  stories: Story[]
  loading: boolean
  error: string | null
  refresh: () => void
}

export function useStories(): UseStoriesResult {
  const [stories, setStories] = useState<Story[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetch = useCallback(() => {
    setLoading(true)
    setError(null)
    listStories()
      .then(setStories)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    fetch()
  }, [fetch])

  return { stories, loading, error, refresh: fetch }
}
