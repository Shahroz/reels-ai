import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  server: {
    // Optional: Configure proxy if backend runs on a different port during development
    // proxy: {
    //   '/api': {
    //     target: 'http://localhost:8080', // Your backend address
    //     changeOrigin: true,
    //     // rewrite: (path) => path.replace(/^\/api/, '') // If backend doesn't expect /api prefix
    //   },
    //   // Proxy WebSocket connections if needed (less common with direct WS URLs)
    //   // '/ws': {
    //   //   target: 'ws://localhost:8080',
    //   //   ws: true,
    //   // },
    // }
  }
})

