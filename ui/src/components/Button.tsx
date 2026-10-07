import type { ButtonHTMLAttributes } from 'react'

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant
  loading?: boolean
}

const variantClasses: Record<Variant, string> = {
  primary:
    'bg-g-accent text-g-bg font-medium hover:bg-amber-400 disabled:opacity-50',
  secondary:
    'border border-g-border text-g-text hover:bg-g-surface-2 disabled:opacity-50',
  ghost:
    'text-g-muted hover:text-g-text hover:bg-g-surface-2 disabled:opacity-50',
  danger:
    'border border-g-danger/40 text-g-danger hover:bg-g-danger/10 disabled:opacity-50',
}

export function Button({
  variant = 'secondary',
  loading = false,
  disabled,
  children,
  className = '',
  ...props
}: ButtonProps) {
  return (
    <button
      disabled={disabled || loading}
      className={[
        'inline-flex items-center gap-2 px-4 py-2 rounded text-sm transition-colors cursor-pointer',
        variantClasses[variant],
        className,
      ].join(' ')}
      {...props}
    >
      {loading && (
        <span className="w-3.5 h-3.5 border-2 border-current border-t-transparent rounded-full animate-spin" />
      )}
      {children}
    </button>
  )
}
