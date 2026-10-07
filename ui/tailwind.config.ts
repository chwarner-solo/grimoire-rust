import type { Config } from 'tailwindcss'

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        g: {
          bg:          '#0f0f11',
          surface:     '#18181b',
          'surface-2': '#27272a',
          border:      '#3f3f46',
          text:        '#f4f4f5',
          muted:       '#a1a1aa',
          accent:      '#f59e0b',
          'accent-dim':'#b45309',
          danger:      '#ef4444',
        },
      },
      fontFamily: {
        sans:  ['Inter', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        serif: ['Lora', 'ui-serif', 'Georgia', 'serif'],
      },
    },
  },
  plugins: [],
} satisfies Config
