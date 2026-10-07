import { get, post } from './client'
import type { Story, StoryId, StoryCreateRequest } from '@/types'

export function listStories(): Promise<Story[]> {
  return get<Story[]>('/dm/stories')
}

export function getStory(id: StoryId): Promise<Story> {
  return get<Story>(`/dm/stories/${id}`)
}

export function createStory(req: StoryCreateRequest): Promise<{ id: StoryId }> {
  return post<{ id: StoryId }>('/dm/stories', req)
}
