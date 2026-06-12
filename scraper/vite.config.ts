import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { fileURLToPath, URL } from 'node:url';

const r = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      '$lib':      r('./src/lib'),
      '$features': r('./src/features'),
      '$ui':       r('./src/ui'),
      '$styles':   r('./src/styles'),
    },
  },
  clearScreen: false,
  server: {
    port: 5174,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
