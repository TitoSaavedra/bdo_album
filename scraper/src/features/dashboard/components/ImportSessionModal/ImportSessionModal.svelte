<script lang="ts">
  import './ImportSessionModal.scss';
  import { invoke } from '@tauri-apps/api/core';
  import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
  import { Button } from '$ui/index';

  let { open = $bindable(false) }: { open: boolean } = $props();

  let hasSession  = $state<boolean | null>(null);
  let filePath    = $state<string | null>(null);
  let processing  = $state(false);
  let success     = $state(false);
  let error       = $state<string | null>(null);

  $effect(() => {
    if (!open) return;
    filePath   = null;
    success    = false;
    error      = null;
    invoke<boolean>('get_garmoth_session_status')
      .then(v => { hasSession = v; })
      .catch(() => { hasSession = null; });
  });

  async function browse() {
    try {
      const selected = await dialogOpen({
        multiple: false,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      });
      if (!selected) return;
      filePath = Array.isArray(selected) ? selected[0] : selected;
      success  = false;
      error    = null;
    } catch (_) {}
  }

  function close() {
    if (processing) return;
    open = false;
  }

  async function runImport() {
    if (!filePath || processing) return;
    processing = true;
    error      = null;

    try {
      await invoke('import_garmoth_session', { path: filePath });
      success     = true;
      hasSession  = true;
      filePath    = null;
    } catch (e) {
      error = String(e);
    } finally {
      processing = false;
    }
  }
</script>

{#if open}
  <div class="pab-backdrop" onclick={close} role="presentation">
    <div class="pab-modal" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="Import Garmoth session" tabindex="-1">

      <div class="pab-header">
        <span class="pab-title">Sesión de Garmoth</span>
        <button class="pab-close" onclick={close} disabled={processing}>✕</button>
      </div>

      <div class="session-body">
        <p class="session-hint">
          Exportá las cookies de <code>garmoth.com</code> con la extensión
          <strong>Cookie-Editor</strong> (botón "Export" → "Export as JSON") estando logueado
          con Discord, y seleccioná ese archivo acá. El scraper usa esa sesión para descargar
          PABs de los presets marcados en el álbum.
        </p>

        <div class="session-status" class:ok={hasSession === true}>
          {#if hasSession === true}
            <span class="status-icon">✓</span> Sesión importada
          {:else if hasSession === false}
            <span class="status-icon">–</span> Sin sesión importada
          {/if}
        </div>

        <button class="drop-browse" onclick={browse} disabled={processing}>
          {filePath ? filePath.split(/[\\/]/).pop() : 'Elegir archivo JSON'}
        </button>

        {#if success}
          <div class="pab-result">
            <div class="result-row success">
              <span class="result-icon">✓</span>
              <span class="result-label">Sesión importada correctamente</span>
            </div>
          </div>
        {/if}

        {#if error}
          <div class="pab-error">{error}</div>
        {/if}
      </div>

      <div class="pab-footer">
        <Button variant="ghost" onclick={close} disabled={processing}>Close</Button>
        <Button
          variant="success"
          onclick={runImport}
          disabled={!filePath || processing}
        >
          {processing ? 'Importando…' : 'Importar sesión'}
        </Button>
      </div>

    </div>
  </div>
{/if}
