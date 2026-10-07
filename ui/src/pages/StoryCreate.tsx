import { useState, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { createStory } from '@/api/stories'
import { ApiRequestError } from '@/api/client'
import { Field } from '@/components/Field'
import { Button } from '@/components/Button'

interface FormState {
  title: string
  prose: string
}

interface FormErrors {
  title?: string
  prose?: string
  submit?: string
}

function validate(form: FormState): FormErrors {
  const errors: FormErrors = {}
  if (!form.title.trim()) errors.title = 'A story needs a name.'
  else if (form.title.trim().length < 2) errors.title = 'Title must be at least 2 characters.'
  return errors
}

export function StoryCreate() {
  const navigate = useNavigate()

  const [form, setForm] = useState<FormState>({ title: '', prose: '' })
  const [errors, setErrors] = useState<FormErrors>({})
  const [submitting, setSubmitting] = useState(false)

  const handleChange = useCallback(
    (field: keyof FormState) =>
      (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
        setForm(prev => ({ ...prev, [field]: e.target.value }))
        setErrors(prev => ({ ...prev, [field]: undefined }))
      },
    [],
  )

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault()
      const errs = validate(form)
      if (Object.keys(errs).length) {
        setErrors(errs)
        return
      }

      setSubmitting(true)
      setErrors({})
      try {
        const { id } = await createStory({
          title: form.title.trim(),
          prose: form.prose.trim(),
        })
        navigate(`/stories/${id}`, { replace: true })
      } catch (err) {
        const message =
          err instanceof ApiRequestError
            ? err.body.message
            : 'Something went wrong. Please try again.'
        setErrors({ submit: message })
      } finally {
        setSubmitting(false)
      }
    },
    [form, navigate],
  )

  return (
    <div className="max-w-xl">
      <div className="mb-8">
        <h1 className="font-serif text-2xl font-medium text-g-text">Begin a new story</h1>
        <p className="text-sm text-g-muted mt-1">
          You can add characters, quests, and sessions later. Just give it a name to start.
        </p>
      </div>

      <form onSubmit={handleSubmit} className="flex flex-col gap-6" noValidate>
        <Field
          label="Title"
          required
          placeholder="The Shattered Crown"
          value={form.title}
          onChange={handleChange('title')}
          error={errors.title}
          autoFocus
        />

        <Field
          as="textarea"
          label="Premise"
          hint="Optional. A sentence or two about the world or the stakes."
          placeholder="In the ruins of a fallen empire, three factions vie for an ancient throne..."
          rows={4}
          value={form.prose}
          onChange={handleChange('prose')}
          error={errors.prose}
        />

        {errors.submit && (
          <p className="text-sm text-g-danger">{errors.submit}</p>
        )}

        <div className="flex gap-3 pt-2">
          <Button type="submit" variant="primary" loading={submitting}>
            Create story
          </Button>
          <Button
            type="button"
            variant="ghost"
            onClick={() => navigate(-1)}
            disabled={submitting}
          >
            Cancel
          </Button>
        </div>
      </form>
    </div>
  )
}
