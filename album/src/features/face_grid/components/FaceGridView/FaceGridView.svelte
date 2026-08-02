<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import {
    getLoading,
    getError,
    getAccounts,
    getActiveAccountId,
    getShowAccountPicker,
    getShowPresetPicker,
    getShowSavePicker,
    getActiveAccount,
    getSavedGrids,
    loadAccounts,
    openSavePicker,
    openAccountPicker,
    openPresetPicker,
  } from '../../state/face_grid.svelte';
  import CharacterGrid from '../CharacterGrid/CharacterGrid.svelte';
  import Dialog from '../Dialog/Dialog.svelte';
  import GridAccountPicker from '../GridAccountPicker/GridAccountPicker.svelte';
  import GridPresetPicker  from '../GridPresetPicker/GridPresetPicker.svelte';

  onMount(async () => {
    await loadAccounts();
  });
</script>

<div class="face-grid-view">
  <div class="fg-body">

    <div class="fg-content">
      {#if getLoading()}
        <div class="fg-state">
          <div class="fg-state-dot"></div>
          <span>{$_('face_grid.view.reading_config')}</span>
        </div>
      {:else if getError()}
        <div class="fg-state fg-state-error">{getError()}</div>
      {:else if getAccounts().length === 0}
        <div class="fg-state">
          <span>{$_('face_grid.view.no_accounts')}</span>
          <span class="fg-state-hint">{$_('face_grid.view.no_accounts_hint')}</span>
        </div>
      {:else if getActiveAccount()}
        <CharacterGrid characters={getActiveAccount()!.characters} />
      {/if}

      <!-- Floating switcher (top-left) -->
      {#if getActiveAccountId()}
        <div class="fg-float-switcher">
          <div class="fg-float-trigger">
            <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="7" cy="4.5" r="2.2"/>
              <path d="M1.5 12c0-2.5 2.5-4.5 5.5-4.5s5.5 2 5.5 4.5"/>
            </svg>
            <span>{getActiveAccountId()}</span>
            <svg width="9" height="9" class="chevron" viewBox="0 0 10 6" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M1 1l4 4 4-4"/>
            </svg>
          </div>
          <div class="fg-float-menu">
            <button class="fg-float-menu-item" onclick={openAccountPicker}>
              <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="7" cy="4.5" r="2.2"/>
                <path d="M1.5 12c0-2.5 2.5-4.5 5.5-4.5s5.5 2 5.5 4.5"/>
              </svg>
              <span>{$_('face_grid.view.accounts')}</span>
              {#if getAccounts().length > 0}
                <span class="menu-badge">{getAccounts().length}</span>
              {/if}
            </button>
            <button class="fg-float-menu-item" onclick={openPresetPicker}>
              <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
                <rect x="1" y="1" width="5" height="5" rx="1"/>
                <rect x="8" y="1" width="5" height="5" rx="1"/>
                <rect x="1" y="8" width="5" height="5" rx="1"/>
                <rect x="8" y="8" width="5" height="5" rx="1"/>
              </svg>
              <span>{$_('face_grid.view.presets')}</span>
              {#if getSavedGrids().length > 0}
                <span class="menu-badge">{getSavedGrids().length}</span>
              {/if}
            </button>
          </div>
        </div>
      {/if}

      <!-- Floating action group (bottom-right) -->
      {#if getActiveAccount()}
        <div class="fg-fab-group">
          <button class="fg-fab-icon" onclick={loadAccounts} title={$_('face_grid.view.reload_title')}>
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M13.5 8A5.5 5.5 0 1 1 8 2.5c1.8 0 3.4.87 4.4 2.2"/>
              <path d="M13.5 2.5v2.5H11"/>
            </svg>
          </button>
          <button class="fg-fab-primary" onclick={openSavePicker}>
            <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
              <rect x="1.5" y="1.5" width="11" height="11" rx="1.5"/>
              <rect x="4" y="1.5" width="6" height="3.5" rx="0.5"/>
              <rect x="3" y="8" width="8" height="3.5" rx="0.5"/>
            </svg>
            {$_('face_grid.view.save_preset')}
          </button>
        </div>
      {/if}
    </div>

  </div>
</div>

<Dialog />
{#if getShowAccountPicker()}
  <GridAccountPicker />
{/if}
{#if getShowPresetPicker()}
  <GridPresetPicker mode="view" />
{/if}
{#if getShowSavePicker()}
  <GridPresetPicker mode="save" />
{/if}

<style lang="scss">
  @use './FaceGridView.scss';
</style>
