<script lang="ts">
  import './ImportSessionModal.scss';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$ui/index';

  let { open = $bindable(false) }: { open: boolean } = $props();

  let hasSession  = $state<boolean | null>(null);
  let jsonText    = $state('');
  let processing  = $state(false);
  let success     = $state(false);
  let error       = $state<string | null>(null);

  let preEl: HTMLElement | undefined = $state();
  let textareaEl: HTMLTextAreaElement | undefined = $state();

  function escapeHtml(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  // Poor-man's JSON syntax highlighting: escape first, then wrap tokens in
  // colored spans. Rendered in a <pre> sitting behind a transparent-text
  // textarea so the caret/selection stay native while the text reads as JSON.
  const highlighted = $derived.by(() => {
    const escaped = escapeHtml(jsonText);
    const html = escaped.replace(
      /"(?:\\u[a-fA-F0-9]{4}|\\[^u]|[^\\"])*"(?:\s*:)?|\btrue\b|\bfalse\b|\bnull\b|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/g,
      (match) => {
        let cls = 'json-number';
        if (match.startsWith('"')) {
          cls = match.endsWith(':') ? 'json-key' : 'json-string';
        } else if (match === 'true' || match === 'false') {
          cls = 'json-boolean';
        } else if (match === 'null') {
          cls = 'json-null';
        }
        return `<span class="${cls}">${match}</span>`;
      },
    );
    return html + '\n';
  });

  function syncScroll() {
    if (preEl && textareaEl) {
      preEl.scrollTop  = textareaEl.scrollTop;
      preEl.scrollLeft = textareaEl.scrollLeft;
    }
  }

  $effect(() => {
    if (!open) return;
    jsonText   = '';
    success    = false;
    error      = null;
    invoke<boolean>('get_garmoth_session_status')
      .then(v => { hasSession = v; })
      .catch(() => { hasSession = null; });
  });

  function close() {
    if (processing) return;
    open = false;
  }

  async function runImport() {
    if (!jsonText.trim() || processing) return;
    processing = true;
    error      = null;

    try {
      await invoke('import_garmoth_session', { json: jsonText });
      success     = true;
      hasSession  = true;
      jsonText    = '';
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
          con Discord — eso copia el JSON al portapapeles — y pegalo acá abajo (Ctrl+V). El
          scraper usa esa sesión para descargar PABs de los presets marcados en el álbum.
        </p>

        <div class="session-status" class:ok={hasSession === true}>
          {#if hasSession === true}
            <span class="status-icon">✓</span> Sesión importada
          {:else if hasSession === false}
            <span class="status-icon">–</span> Sin sesión importada
          {/if}
        </div>

        <div class="session-json-editor">
          <pre class="session-json-highlight" bind:this={preEl} aria-hidden="true"><code>{@html jsonText ? highlighted : ''}</code></pre>
          <textarea
            class="session-json-input"
            placeholder="Pegá acá el JSON exportado por Cookie-Editor…"
            bind:value={jsonText}
            bind:this={textareaEl}
            onscroll={syncScroll}
            oninput={syncScroll}
            disabled={processing}
            spellcheck="false"
            rows="8"
          ></textarea>
        </div>

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
          disabled={!jsonText.trim() || processing}
        >
          {processing ? 'Importando…' : 'Importar sesión'}
        </Button>
      </div>

    </div>
  </div>
{/if}
