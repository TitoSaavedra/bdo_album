<script lang="ts">
  import './RepairPabModal.scss';
  import { invoke } from '@tauri-apps/api/core';
  import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
  import { Button } from '$ui/index';

  let { open = $bindable(false) }: { open: boolean } = $props();

  interface PabSearchResult {
    preset_id:      number;
    character_name: string | null;
    class_name:     string;
    url:            string;
    synced_at:      number;
  }

  type RowStatus =
    | { kind: 'idle' }
    | { kind: 'repairing' }
    | { kind: 'repaired'; db_path: string }
    | { kind: 'error'; message: string };

  let query        = $state('');
  let results      = $state<PabSearchResult[]>([]);
  let searching    = $state(false);
  let searched     = $state(false);
  let repairingId  = $state<number | null>(null);
  let rowStatus    = $state<Record<number, RowStatus>>({});
  let searchError  = $state<string | null>(null);

  let debounceHandle: ReturnType<typeof setTimeout> | undefined;

  function onQueryInput() {
    clearTimeout(debounceHandle);
    debounceHandle = setTimeout(runSearch, 300);
  }

  async function runSearch() {
    searching   = true;
    searchError = null;
    try {
      results  = await invoke<PabSearchResult[]>('search_repairable_pabs', { query });
      searched = true;
    } catch (e) {
      searchError = String(e);
    } finally {
      searching = false;
    }
  }

  async function repair(presetId: number) {
    if (repairingId !== null) return;

    let path: string | string[] | null;
    try {
      path = await dialogOpen({
        multiple: false,
        filters: [{ name: 'PAB', extensions: ['pab'] }],
      });
    } catch (_) {
      return;
    }
    if (!path || Array.isArray(path)) return;

    repairingId = presetId;
    rowStatus = { ...rowStatus, [presetId]: { kind: 'repairing' } };

    try {
      const res = await invoke<{ db_path: string }>('repair_pab', { presetId, path });
      rowStatus = { ...rowStatus, [presetId]: { kind: 'repaired', db_path: res.db_path } };
    } catch (e) {
      rowStatus = { ...rowStatus, [presetId]: { kind: 'error', message: String(e) } };
    } finally {
      repairingId = null;
    }
  }

  function close() {
    if (repairingId !== null) return;
    open        = false;
    query       = '';
    results     = [];
    searched    = false;
    rowStatus   = {};
    searchError = null;
  }

  function formatDate(ms: number): string {
    return new Date(ms).toLocaleString();
  }
</script>

{#if open}
  <div class="repair-backdrop" onclick={close} role="presentation">
    <div class="repair-modal" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="Repair PAB" tabindex="-1">

      <!-- Header -->
      <div class="repair-header">
        <span class="repair-title">Reparar PAB</span>
        <button class="repair-close" onclick={close} disabled={repairingId !== null}>✕</button>
      </div>

      <div class="repair-hint">
        Busca el preset y elegí el <code>.pab</code> original que ya tenés en disco (el que importaste
        a mano o el que Garmoth te dejó en Descargas). Lo sube <strong>sin</strong> el parche que lo
        marca como editable — ese parche es lo que puede corromper el archivo en algunas clases.
      </div>

      <!-- Search -->
      <div class="repair-search">
        <input
          type="text"
          placeholder="Buscar por ID de preset, personaje o clase…"
          bind:value={query}
          oninput={onQueryInput}
          disabled={repairingId !== null}
        />
      </div>

      <!-- Results -->
      <div class="repair-results">
        {#if searching}
          <div class="repair-empty">Buscando…</div>
        {:else if searchError}
          <div class="repair-error">{searchError}</div>
        {:else if searched && results.length === 0}
          <div class="repair-empty">Sin resultados</div>
        {:else if !searched}
          <div class="repair-empty">Escribe para buscar presets con un PAB ya subido</div>
        {:else}
          {#each results as row (row.preset_id)}
            {@const status = rowStatus[row.preset_id] ?? { kind: 'idle' }}
            <div class="repair-row">
              <div class="repair-row-info">
                <span class="repair-row-name">{row.character_name ?? `Preset ${row.preset_id}`}</span>
                <span class="repair-row-meta">{row.class_name} · ID {row.preset_id} · sync {formatDate(row.synced_at)}</span>
              </div>

              <div class="repair-row-action">
                {#if status.kind === 'idle'}
                  <Button variant="ghost" onclick={() => repair(row.preset_id)} disabled={repairingId !== null}>
                    Elegir archivo…
                  </Button>
                {:else if status.kind === 'repairing'}
                  <span class="repair-status pending">Subiendo…</span>
                {:else if status.kind === 'repaired'}
                  <span class="repair-status success">✓ Reparado</span>
                {:else if status.kind === 'error'}
                  <span class="repair-status error" title={status.message}>Error</span>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      </div>

      <!-- Footer -->
      <div class="repair-footer">
        <Button variant="ghost" onclick={close} disabled={repairingId !== null}>Close</Button>
      </div>

    </div>
  </div>
{/if}
