// Mirror of domain types — keep in sync with docs/openapi.yaml

export type StoryId = string
export type CharacterId = string
export type QuestId = string
export type LocationId = string
export type SessionId = string
export type EncounterId = string

export interface Story {
  id: StoryId
  owner: string
  title: string
  prose: string
  status: 'Active' | 'Inactive'
}

export interface StoryCreateRequest {
  title: string
  prose: string
}

export interface ApiError {
  error: string
  message: string
}
