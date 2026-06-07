<script lang="ts">
  import { closeSaveDialog, saveGrid } from '../../state/face_grid.svelte';

  let name    = $state('');
  let saving  = $state(false);
  let error   = $state<string | null>(null);

  async function handleSave() {
    if (!name.trim()) return;
    saving = true;
    error  = null;
    try {
      await saveGrid(name.trim());
      name = '';
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleSave();
    if (e.key === 'Escape') closeSaveDialog();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="dialog-backdrop" onclick={closeSaveDialog}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <h3>Guardar grid</h3>

    <input
      type="text"
      placeholder="Nombre del grid"
      bind:value={name}
      {onkeydown}
      autofocus
    />

    {#if error}
      <div style="color:#c44;font-size:0.8rem">{error}</div>
    {/if}

    <div class="actions">
      <button class="cancel" onclick={closeSaveDialog}>Cancelar</button>
      <button class="save" disabled={!name.trim() || saving} onclick={handleSave}>
        {saving ? 'Guardando...' : 'Guardar'}
      </button>
    </div>
  </div>
</div>
