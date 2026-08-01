<script lang="ts">
  import './ImportPabModal.scss';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
  import { Button } from '$ui/index';

  let { open = $bindable(false) }: { open: boolean } = $props();

  interface PabFile { name: string; path: string; }
  interface ImportResult {
    found:           number;
    patched:         number;
    not_found_count: number;
    not_found_names: string[];
    zip_base64:      string | null;
  }

  let files      = $state<PabFile[]>([]);
  let isDragging = $state(false);
  let processing = $state(false);
  let result     = $state<ImportResult | null>(null);
  let error      = $state<string | null>(null);

  let _unlisteners: UnlistenFn[] = [];

  $effect(() => {
    if (!open) {
      _unlisteners.forEach(fn => fn());
      _unlisteners = [];
      return;
    }

    Promise.all([
      listen<{ paths: string[] }>('tauri://drag-enter', () => { isDragging = true; }),
      listen<{ paths: string[] }>('tauri://drag-drop',  (e) => { isDragging = false; addPaths(e.payload.paths); }),
      listen<void>('tauri://drag-leave',                 () => { isDragging = false; }),
    ]).then(fns => { _unlisteners = fns; });

    return () => {
      _unlisteners.forEach(fn => fn());
      _unlisteners = [];
    };
  });

  function addPaths(incoming: string[]) {
    const existing = new Set(files.map(f => f.path));
    const fresh = incoming
      .filter(p => !existing.has(p))
      .map(p => ({ path: p, name: p.split(/[\\/]/).pop() ?? p }));
    if (fresh.length === 0) return;
    files  = [...files, ...fresh];
    result = null;
    error  = null;
  }

  async function browse() {
    try {
      const selected = await dialogOpen({ multiple: true });
      if (!selected) return;
      addPaths(Array.isArray(selected) ? selected : [selected]);
    } catch (_) {}
  }

  function removeFile(i: number) { files = files.filter((_, idx) => idx !== i); }

  function close() {
    if (processing) return;
    open       = false;
    files      = [];
    result     = null;
    error      = null;
    isDragging = false;
  }

  async function runImport() {
    if (files.length === 0 || processing) return;
    processing = true;
    error      = null;
    result     = null;

    try {
      const res = await invoke<ImportResult>('import_pab_files', {
        paths: files.map(f => f.path),
      });
      result = res;

      if (res.zip_base64) {
        const raw  = atob(res.zip_base64);
        const arr  = Uint8Array.from(raw, c => c.charCodeAt(0));
        const blob = new Blob([arr], { type: 'application/zip' });
        const url  = URL.createObjectURL(blob);
        const a    = document.createElement('a');
        a.href     = url;
        a.download = 'not_found_presets.zip';
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
      }
    } catch (e) {
      error = String(e);
    } finally {
      processing = false;
    }
  }
</script>

{#if open}
  <div class="pab-backdrop" onclick={close} role="presentation">
    <div class="pab-modal" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="Import PAB files" tabindex="-1">

      <!-- Loading bar -->
      {#if processing}
        <div class="pab-progress-bar"><div class="pab-progress-fill"></div></div>
      {/if}

      <!-- Header -->
      <div class="pab-header">
        <span class="pab-title">Import PAB Files</span>
        <button class="pab-close" onclick={close} disabled={processing}>✕</button>
      </div>

      <!-- Drop zone — hidden while processing or done -->
      {#if !processing && !result}
        <div class="pab-drop-zone" class:dragging={isDragging} role="presentation">
          <span class="drop-icon">⬇</span>
          <span class="drop-label">Drag PAB files from Explorer</span>
          <span class="drop-hint">Matched by <code>ID&lt;digits&gt;_</code> in the filename</span>
          <button class="drop-browse" onclick={browse}>Browse files</button>
        </div>
      {/if}

      <!-- Processing state -->
      {#if processing}
        <div class="pab-processing">
          <span class="processing-label">Uploading to R2…</span>
          <span class="processing-count">{files.length} file{files.length !== 1 ? 's' : ''}</span>
        </div>
      {/if}

      <!-- File list — hidden while processing or done -->
      {#if !processing && !result && files.length > 0}
        <div class="pab-file-list">
          <div class="file-list-header">
            <span class="file-list-count">{files.length} file{files.length !== 1 ? 's' : ''} selected</span>
            <button class="file-clear-all" onclick={() => { files = []; result = null; }}>Clear all</button>
          </div>
          <div class="file-list-scroll">
            {#each files as file, i}
              <div class="file-row">
                <span class="file-name" title={file.path}>{file.name}</span>
                <button class="file-remove" onclick={() => removeFile(i)}>✕</button>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Result -->
      {#if result}
        <div class="pab-result">
          <div class="result-row success">
            <span class="result-icon">✓</span>
            <span class="result-label">{result.found} preset{result.found !== 1 ? 's' : ''} imported to R2</span>
          </div>
          {#if result.patched > 0}
            <div class="result-row success">
              <span class="result-icon">✎</span>
              <span class="result-label">{result.patched} archivo{result.patched !== 1 ? 's' : ''} dejado{result.patched !== 1 ? 's' : ''} editable{result.patched !== 1 ? 's' : ''}</span>
            </div>
          {/if}
          {#if result.not_found_count > 0}
            <div class="result-row warn">
              <span class="result-icon">⬇</span>
              <span class="result-label">{result.not_found_count} not found — downloaded to your Downloads folder</span>
            </div>
            <div class="result-not-found-list">
              {#each result.not_found_names as name}
                <span class="nf-name">{name}</span>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Error -->
      {#if error}
        <div class="pab-error">{error}</div>
      {/if}

      <!-- Footer -->
      <div class="pab-footer">
        <Button variant="ghost" onclick={close} disabled={processing}>Close</Button>
        {#if !result}
          <Button
            variant="success"
            onclick={runImport}
            disabled={files.length === 0 || processing}
          >
            {processing ? 'Importing…' : `Import ${files.length > 0 ? files.length + ' file' + (files.length !== 1 ? 's' : '') : ''}`}
          </Button>
        {/if}
      </div>

    </div>
  </div>
{/if}
