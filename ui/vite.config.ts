import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { VitePWA } from 'vite-plugin-pwa'
import { resolve } from 'path'

export default defineConfig({
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  plugins: [
    react(),
    VitePWA({
      registerType: 'autoUpdate',
      manifest: {
        name: 'Grimoire',
        short_name: 'Grimoire',
        description: 'TTRPG campaign manager',
        theme_color: '#0f0f11',
        background_color: '#0f0f11',
        display: 'standalone',
        icons: [
          { src: '/icon-192.png', sizes: '192x192', type: 'image/png' },
          { src: '/icon-512.png', sizes: '512x512', type: 'image/png' },
        ],
      },
    }),
  ],
  server: {
    proxy: {
      '/dm': 'http://localhost:3000',
      '/player': 'http://localhost:3000',
      '/health': 'http://localhost:3000',
    },
  },
})
