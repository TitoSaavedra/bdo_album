<script lang="ts">
  import { onMount } from 'svelte';
  import {
    activeAccount,
    clearAllSlots,
    faceGrid,
    loadAccounts,
    openSaveDialog,
  } from '../../state/face_grid.svelte';
  import AccountTabs from '../AccountTabs/AccountTabs.svelte';
  import CharacterGrid from '../CharacterGrid/CharacterGrid.svelte';
  import FaceGridSidebar from '../FaceGridSidebar/FaceGridSidebar.svelte';
  import PresetPicker from '../PresetPicker/PresetPicker.svelte';
  import SaveGridDialog from '../SaveGridDialog/SaveGridDialog.svelte';

  onMount(() => loadAccounts());

  const account       = $derived(activeAccount());
  const pendingCount  = $derived(Object.keys(faceGrid.pendingSlots).length);
</script>

<div class="face-grid-view">
  <div class="fg-main">
    <FaceGridSidebar />

    <div class="fg-center">
      {#if faceGrid.loading}
        <div class="fg-loading">Leyendo configuración de BDO...</div>
      {:else if faceGrid.error}
        <div class="fg-error">{faceGrid.error}</div>
      {:else if faceGrid.accounts.length === 0}
        <div class="fg-empty">
          <span>No se encontraron cuentas de BDO</span>
          <span style="font-size:0.75rem;color:#333">
            Verifica que BDO esté instalado y hayas iniciado sesión al menos una vez
          </span>
        </div>
      {:else}
        <AccountTabs accounts={faceGrid.accounts} />

        <div class="fg-toolbar">
          {#if pendingCount > 0}
            <span class="pending-count">{pendingCount} cambio{pendingCount > 1 ? 's' : ''} pendiente{pendingCount > 1 ? 's' : ''}</span>
            <button class="btn-clear" onclick={clearAllSlots}>Limpiar</button>
            <button class="btn-save" onclick={openSaveDialog}>Guardar grid</button>
          {/if}
        </div>

        {#if account}
          <CharacterGrid characters={account.characters} />
        {/if}
      {/if}
    </div>

    <PresetPicker />
  </div>

  {#if faceGrid.saveDialogOpen}
    <SaveGridDialog />
  {/if}
</div>
