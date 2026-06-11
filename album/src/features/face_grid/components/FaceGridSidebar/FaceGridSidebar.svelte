<script lang="ts">
  import './FaceGridSidebar.scss';
  import Button from '../../../../ui/Button/Button.svelte';
  import { applyGrid, deleteGrid, faceGrid, openConfirmDialog } from '../../state/face_grid.svelte';

  function handleApply(id: number) {
    openConfirmDialog(
      'Apply Grid',
      'FaceTexture images will be overwritten. Continue?',
      () => applyGrid(id),
      'Apply'
    );
  }

  function handleDelete(id: number) {
    openConfirmDialog(
      'Delete Grid',
      'Are you sure? This action cannot be undone.',
      () => deleteGrid(id),
      'Delete'
    );
  }
</script>

<aside class="sidebar-grids">
  <div class="sidebar-heading">Saved Grids</div>

  <div class="grids-list custom-scroll">
    {#if faceGrid.savedGrids.length === 0}
      <div class="sidebar-empty">No saved grids</div>
    {:else}
      {#each faceGrid.savedGrids as grid (grid.id)}
        <div class="grid-item" class:active={faceGrid.activeGridId === grid.id}>
          <div class="item-info">
            <div class="item-title">{grid.name}</div>
            <div class="item-subtitle">Account {grid.account_id}</div>
          </div>

          <div class="item-actions">
            <Button
              variant="primary"
              disabled={faceGrid.applyingGrid}
              onclick={() => handleApply(grid.id)}
            >
              Apply
            </Button>
            <Button
              variant="danger"
              disabled={faceGrid.applyingGrid}
              onclick={() => handleDelete(grid.id)}
            >
              Delete
            </Button>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</aside>

<style lang="scss">
  @use './FaceGridSidebar.scss';
</style>
