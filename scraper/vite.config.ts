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
    watch: {
      // Cargo build output now lives at the workspace root (scraper/target/)
      // instead of nested under src-tauri/ since the core/cli workspace split
      // — without also ignoring it here, Vite's fs watcher races cargo writing
      // to target/debug/deps/*.dll mid-build and crashes with EBUSY.
      ignored: ['**/src-tauri/**', '**/target/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
