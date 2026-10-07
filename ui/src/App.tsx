import { Routes, Route, Navigate } from 'react-router-dom'
import { Layout } from '@/components/Layout'
import { StoryList } from '@/pages/StoryList'
import { StoryCreate } from '@/pages/StoryCreate'

export function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Navigate to="/stories" replace />} />
        <Route path="/stories" element={<StoryList />} />
        <Route path="/stories/new" element={<StoryCreate />} />
        <Route path="*" element={<Navigate to="/stories" replace />} />
      </Routes>
    </Layout>
  )
}
