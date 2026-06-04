# BDO Album

Dos proyectos Tauri/Rust completamente independientes. Sin crates compartidos, sin workspace.

---

## Proyectos

### `scrapper/` — Dashboard + Scrapper
- Descarga presets populares de Garmoth
- Sube imágenes a Cloudflare R2
- Guarda metadata en PostgreSQL
- Muestra progreso y logs en tiempo real
- **Dev**: `cd scrapper && pnpm tauri dev`

### `album/` — Beauty Album Viewer
- Muestra presets descargados por el scrapper
- Lee de la misma PostgreSQL
- Gestiona preferencias del usuario (favoritos, descartados)
- **Dev**: `cd album && pnpm tauri dev`

---

## Infraestructura compartida

```bash
docker compose up -d   # PostgreSQL en localhost:5432
```

Ambos proyectos se conectan a la misma DB (`bdo_album`). El scrapper escribe, el album lee.

---

## Reglas de código

- **SCSS**: siempre archivo `.scss` separado. Nunca `<style>` en `.svelte`
- **Svelte 5**: runes (`$state`, `$derived`, `$effect`, `$props`). Sin Svelte 4 stores
- **State**: archivos `.svelte.ts` en `features/<name>/state/`
- **Rust**: commands thin → service → repository. Sin lógica en commands
- **No build/test automático**

---

## Variables de entorno

Cada proyecto tiene su propio `src-tauri/.env` (ignorado por git).

### scrapper/src-tauri/.env
```env
DATABASE_URL=postgresql://bdo:bdo@localhost:5432/bdo_album
R2_ACCOUNT_ID=...
R2_ACCESS_KEY_ID=...
R2_SECRET_ACCESS_KEY=...
R2_BUCKET_NAME=bdo-album
R2_ENDPOINT=https://....r2.cloudflarestorage.com
```

### album/src-tauri/.env
```env
DATABASE_URL=postgresql://bdo:bdo@localhost:5432/bdo_album
```
