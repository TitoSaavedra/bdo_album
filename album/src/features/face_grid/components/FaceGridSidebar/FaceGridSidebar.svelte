<script lang="ts">
  import { _ } from 'svelte-i18n';
  import {
    getSavedGrids,
    getActiveGridId,
    getApplyingGrid,
    getCreatingPreset,
    getCreateProgress,
    getGridThumb,
    applyGrid,
    deleteGrid,
    openConfirmDialog,
  } from '../../state/face_grid.svelte';

  const CARD_COLORS = [
    ['#1a1a2e', '#16213e'],
    ['#1e1e2e', '#2d1b69'],
    ['#1a2a1a', '#1e3a1e'],
    ['#2a1a1a', '#3a1e1e'],
    ['#1a2a2a', '#1e3a3a'],
    ['#2a2a1a', '#3a3a1e'],
    ['#221a2a', '#381e42'],
  ];

  function cardColors(id: number) {
    const pair = CARD_COLORS[id % CARD_COLORS.length];
    return `background: linear-gradient(135deg, ${pair[0]} 0%, ${pair[1]} 100%)`;
  }

  function handleApply(id: number) {
    openConfirmDialog(
      $_('face_grid.sidebar.apply_grid_title'),
      $_('face_grid.sidebar.apply_grid_msg'),
      () => applyGrid(id),
      $_('face_grid.sidebar.apply_confirm')
    );
  }

  function handleDelete(id: number) {
    openConfirmDialog(
      $_('face_grid.sidebar.delete_grid_title'),
      $_('face_grid.sidebar.delete_grid_msg'),
      () => deleteGrid(id),
      $_('face_grid.sidebar.delete_confirm')
    );
  }
</script>

<aside class="sidebar-grids">
  <div class="sidebar-heading">
    <div class="sidebar-heading-pill">
      <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
        <rect x="1" y="1" width="5" height="5" rx="1"/>
        <rect x="8" y="1" width="5" height="5" rx="1"/>
        <rect x="1" y="8" width="5" height="5" rx="1"/>
        <rect x="8" y="8" width="5" height="5" rx="1"/>
      </svg>
      <span>{$_('face_grid.sidebar.presets')}</span>
      {#if getSavedGrids().length > 0}
        <span class="sidebar-count">{getSavedGrids().length}</span>
      {/if}
    </div>
  </div>

  <div class="grids-list custom-scroll">
    {#if getCreatingPreset()}
      {@const pct = getCreateProgress().total > 0
        ? Math.round((getCreateProgress().current / getCreateProgress().total) * 100)
        : 0}
      <div class="sidebar-creating">
        <span class="creating-label">{$_('face_grid.sidebar.creating_preset', { values: { pct } })}</span>
        <div class="progress-track">
          <div class="progress-fill" style="width: {pct}%"></div>
        </div>
        {#if getCreateProgress().character_no}
          <span class="creating-char">{getCreateProgress().character_no}</span>
        {/if}
      </div>
    {:else if getSavedGrids().length === 0}
      <div class="sidebar-empty">{$_('face_grid.sidebar.no_presets')}</div>
    {:else}
      <div class="cards-grid">
        {#each getSavedGrids() as grid (grid.id)}
          {@const isDefault = grid.name === '_default'}
          {@const isActive  = getActiveGridId() === grid.id}
          {@const thumb     = getGridThumb(grid.account_id)}
          <div class="preset-card" class:active={isActive} class:is-default={isDefault}>
            <div class="card-thumb" style={thumb ? '' : cardColors(grid.id)}>
              {#if thumb}
                <img src={thumb} alt="" class="thumb-img" />
              {/if}
              {#if isDefault}
                <span class="default-badge">{$_('face_grid.sidebar.default')}</span>
              {/if}
              <div class="card-hover-actions">
                <button
                  class="action-btn apply-btn"
                  disabled={getApplyingGrid()}
                  onclick={() => handleApply(grid.id)}
                  title={$_('face_grid.sidebar.apply_title')}
                >
                  {$_('face_grid.sidebar.apply')}
                </button>
                {#if !isDefault}
                  <button
                    class="action-btn delete-btn"
                    disabled={getApplyingGrid()}
                    onclick={() => handleDelete(grid.id)}
                    title={$_('face_grid.sidebar.delete_title')}
                  >
                    &#x2715;
                  </button>
                {/if}
              </div>
            </div>
            <div class="card-footer">
              <span class="card-name" title={grid.name}>{grid.name}</span>
              <span class="card-account">{grid.account_id.slice(-6)}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</aside>

<style lang="scss">
  @use './FaceGridSidebar.scss';
</style>
