import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  base: mode === 'pages' ? '/ApexTelemetry-for-GT/' : '/',
  plugins: [react()],
  server: {
    proxy: {
      '/config': {
        target: 'http://127.0.0.1:10086',
        changeOrigin: true,
      },
      '/debug': {
        target: 'http://127.0.0.1:10086',
        changeOrigin: true,
      },
      '/meta': {
        target: 'http://127.0.0.1:10086',
        changeOrigin: true,
      },
      '/demo': {
        target: 'http://127.0.0.1:10086',
        changeOrigin: true,
      },
    },
  },
}))
