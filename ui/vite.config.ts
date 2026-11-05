import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Конфигурация сборки фронтенда Fontlord.
// Порт 1420 используется как `devUrl` в tauri.conf.json.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'es2021',
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: false,
  },
});
