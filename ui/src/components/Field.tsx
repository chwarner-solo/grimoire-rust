import type { InputHTMLAttributes, TextareaHTMLAttributes } from 'react'

interface FieldProps {
  label: string
  hint?: string
  error?: string
  required?: boolean
}

interface InputFieldProps extends FieldProps, InputHTMLAttributes<HTMLInputElement> {
  as?: 'input'
}

interface TextareaFieldProps extends FieldProps, TextareaHTMLAttributes<HTMLTextAreaElement> {
  as: 'textarea'
}

type Props = InputFieldProps | TextareaFieldProps

const baseInput =
  'w-full bg-g-surface-2 border border-g-border rounded px-3 py-2 text-g-text placeholder:text-g-muted/50 focus:outline-none focus:border-g-accent/60 focus:ring-1 focus:ring-g-accent/30 transition-colors text-sm'

export function Field(props: Props) {
  const { label, hint, error, required, as: Tag = 'input', ...rest } = props

  const id = `field-${label.toLowerCase().replace(/\s+/g, '-')}`

  return (
    <div className="flex flex-col gap-1.5">
      <label htmlFor={id} className="text-sm font-medium text-g-text">
        {label}
        {required && <span className="text-g-accent ml-1">*</span>}
      </label>
      {hint && <p className="text-xs text-g-muted -mt-0.5">{hint}</p>}
      {Tag === 'textarea' ? (
        <textarea
          id={id}
          className={`${baseInput} resize-none`}
          {...(rest as TextareaHTMLAttributes<HTMLTextAreaElement>)}
        />
      ) : (
        <input
          id={id}
          className={baseInput}
          {...(rest as InputHTMLAttributes<HTMLInputElement>)}
        />
      )}
      {error && <p className="text-xs text-g-danger">{error}</p>}
    </div>
  )
}
