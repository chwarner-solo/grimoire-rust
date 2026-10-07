import { Link, useNavigate } from 'react-router-dom'
import { useStories } from '@/hooks/useStories'
import { Button } from '@/components/Button'
import type { Story } from '@/types'

function StatusBadge({ status }: { status: Story['status'] }) {
  const colors = {
    Active:   'text-emerald-400 bg-emerald-400/10 border-emerald-400/30',
    Inactive: 'text-g-muted bg-g-surface-2 border-g-border',
  }
  return (
    <span className={`text-xs px-2 py-0.5 rounded-full border font-medium ${colors[status]}`}>
      {status}
    </span>
  )
}

function StoryCard({ story }: { story: Story }) {
  return (
    <Link
      to={`/stories/${story.id}`}
      className="block p-5 rounded-lg border border-g-border bg-g-surface hover:border-g-accent/40 hover:bg-g-surface-2 transition-all group"
    >
      <div className="flex items-start justify-between gap-4">
        <h2 className="font-serif text-lg text-g-text group-hover:text-g-accent transition-colors leading-snug">
          {story.title}
        </h2>
        <StatusBadge status={story.status} />
      </div>
      {story.prose && (
        <p className="mt-2 text-sm text-g-muted line-clamp-2 leading-relaxed">
          {story.prose}
        </p>
      )}
    </Link>
  )
}

export function StoryList() {
  const { stories, loading, error } = useStories()
  const navigate = useNavigate()

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="font-serif text-2xl font-medium text-g-text">Your Stories</h1>
          <p className="text-sm text-g-muted mt-1">Each story is a campaign. Start one to begin recording history.</p>
        </div>
        <Button variant="primary" onClick={() => navigate('/stories/new')}>
          New story
        </Button>
      </div>

      {loading && (
        <div className="flex justify-center py-16">
          <span className="w-5 h-5 border-2 border-g-accent border-t-transparent rounded-full animate-spin" />
        </div>
      )}

      {error && (
        <div className="p-4 rounded border border-g-danger/30 bg-g-danger/5 text-sm text-g-danger">
          {error}
        </div>
      )}

      {!loading && !error && stories.length === 0 && (
        <div className="text-center py-20">
          <p className="font-serif text-xl text-g-muted">No stories yet.</p>
          <p className="text-sm text-g-muted/60 mt-2">Create your first story to begin.</p>
          <Button variant="primary" className="mt-6" onClick={() => navigate('/stories/new')}>
            Create a story
          </Button>
        </div>
      )}

      {!loading && !error && stories.length > 0 && (
        <div className="flex flex-col gap-3">
          {stories.map(story => (
            <StoryCard key={story.id} story={story} />
          ))}
        </div>
      )}
    </div>
  )
}
