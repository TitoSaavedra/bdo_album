<script lang="ts">
  import { scale } from 'svelte/transition';
  import { _ } from 'svelte-i18n';
  import {
    getSavedGrids,
    getActiveGridId,
    getApplyingGrid,
    getAllGridSlots,
    applyGrid,
    deleteGrid,
    overwriteGrid,
    openConfirmDialog,
    openSaveGridDialog,
    closePresetPicker,
    closeSavePicker,
  } from '../../state/face_grid.svelte';
  import PresetThumb from '../PresetThumb/PresetThumb.svelte';

  let { mode = 'view' }: { mode?: 'view' | 'save' } = $props();

  const isSave   = $derived(mode === 'save');
  const close    = $derived(isSave ? closeSavePicker : closePresetPicker);

  // In save mode, pre-select the active grid
  let selectedId = $state<number | null>(getActiveGridId());

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

  function handleSaveNew() {
    closeSavePicker();
    openSaveGridDialog();
  }

  function handleOverwrite(id: number, name: string) {
    openConfirmDialog(
      $_('face_grid.preset_picker.overwrite_title'),
      $_('face_grid.preset_picker.overwrite_msg', { values: { name } }),
      () => overwriteGrid(id),
      $_('face_grid.preset_picker.overwrite_confirm')
    );
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="picker-backdrop"
  onclick={close}
  onkeydown={(e) => e.key === 'Escape' && close()}
>
  <div
    class="picker-dialog"
    role="dialog"
    tabindex="-1"
    transition:scale={{ duration: 150, start: 0.95 }}
    onclick={(e) => e.stopPropagation()}
  >
    <h3>
      {isSave ? $_('face_grid.preset_picker.save_title') : $_('face_grid.preset_picker.saved_title')}
      {#if getSavedGrids().length > 0}
        <span class="preset-count">{getSavedGrids().length}</span>
      {/if}
    </h3>

    <div class="presets-list custom-scroll">
      {#if isSave}
        <button class="new-preset-card" onclick={handleSaveNew}>
          <span class="new-icon">+</span>
          <span class="new-label">{$_('face_grid.preset_picker.save_as_new')}</span>
        </button>
      {/if}

      {#if getSavedGrids().length === 0}
        <div class="presets-empty">{$_('face_grid.preset_picker.empty')}</div>
      {:else}
        {#each getSavedGrids() as grid (grid.id)}
          {@const isDefault  = grid.name === '_default'}
          {@const isActive   = getActiveGridId() === grid.id}
          {@const isSelected = isSave && selectedId === grid.id}
          {@const slots      = getAllGridSlots()[grid.id] ?? []}
          {@const r2urls     = slots.filter(s => s.image_url).map(s => s.image_url)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            class="preset-item"
            class:active={isActive}
            class:selected={isSelected}
            class:is-default={isDefault}
            onclick={() => isSave && (selectedId = grid.id)}
            role={isSave ? 'button' : undefined}
            tabindex={isSave ? 0 : undefined}
          >
            <div class="item-thumb" style={r2urls.length === 0 ? cardColors(grid.id) : ''}>
              <PresetThumb urls={r2urls} />
              {#if isDefault}
                <span class="default-badge">{$_('face_grid.sidebar.default')}</span>
              {/if}
              {#if isSelected}
                <div class="selected-overlay">
                  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="20 6 9 17 4 12"/>
                  </svg>
                </div>
              {/if}
            </div>
            <div class="item-info">
              <span class="item-name" title={grid.name}>{grid.name}</span>
              <span class="item-account">{grid.account_id.slice(-6)}</span>
            </div>
            {#if !isSave}
              <div class="item-actions">
                <button
                  class="action-btn apply-btn"
                  disabled={getApplyingGrid()}
                  onclick={() => handleApply(grid.id)}
                >{$_('face_grid.sidebar.apply')}</button>
                {#if !isDefault}
                  <button
                    class="action-btn delete-btn"
                    disabled={getApplyingGrid()}
                    onclick={() => handleDelete(grid.id)}
                  >&#x2715;</button>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    {#if isSave}
      <div class="save-footer">
        <button class="footer-cancel" onclick={closeSavePicker}>{$_('face_grid.preset_picker.cancel')}</button>
        <button
          class="footer-save"
          disabled={selectedId === null}
          onclick={() => selectedId !== null && handleOverwrite(
            selectedId,
            getSavedGrids().find(g => g.id === selectedId)?.name ?? ''
          )}
        >
          {selectedId !== null
            ? $_('face_grid.preset_picker.overwrite', { values: { name: getSavedGrids().find(g => g.id === selectedId)?.name ?? '' } })
            : $_('face_grid.preset_picker.select_a_preset')}
        </button>
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
  @use './GridPresetPicker.scss';
</style>
