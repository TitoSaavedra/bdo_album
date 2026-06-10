<script lang="ts">
  import './FaceGridSidebar.scss';
  import { applyGrid, deleteGrid, faceGrid } from '../../state/face_grid.svelte';

  async function handleApply(gridId: string) {
    if (!confirm('¿Aplicar este grid? Se sobreescribirán las imágenes en FaceTexture.')) return;
    await applyGrid(gridId);
  }

  async function handleDelete(gridId: string) {
    if (!confirm('¿Eliminar este grid?')) return;
    await deleteGrid(gridId);
  }
</script>

<aside class="face-sidebar">
  <div class="sidebar-header">Grids guardados</div>

  <div class="grid-list">
    {#if faceGrid.savedGrids.length === 0}
      <div class="empty-grids">Sin grids guardados</div>
    {:else}
      {#each faceGrid.savedGrids as grid (grid.id)}
        <div class="grid-item">
          {#if grid.thumbnail_url}
            <img src={grid.thumbnail_url} alt="" class="grid-thumb" />
          {:else}
            <div class="thumb-placeholder">sin thumbnail</div>
          {/if}
          <div class="grid-meta">
            <div class="grid-name">{grid.name}</div>
            <div class="grid-account">Cuenta {grid.account_id}</div>
          </div>
          <div class="grid-actions">
            <button
              class="apply-btn"
              disabled={faceGrid.applyingGrid}
              onclick={() => handleApply(grid.id)}
            >
              {faceGrid.applyingGrid ? '...' : 'Aplicar'}
            </button>
            <button class="delete-btn" onclick={() => handleDelete(grid.id)}>
              Eliminar
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</aside>
