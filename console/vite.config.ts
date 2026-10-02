import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  base: "./",
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      '/v1': { target: 'http://127.0.0.1:8080', changeOrigin: false },
      '/health': { target: 'http://127.0.0.1:8080', changeOrigin: false },
      '/ready': { target: 'http://127.0.0.1:8080', changeOrigin: false },
    },
  },
  test: { environment: 'node', include: ['src/**/*.test.ts'] },
});
