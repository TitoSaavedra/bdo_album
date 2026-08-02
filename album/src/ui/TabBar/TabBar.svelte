<script lang="ts">
  import './TabBar.scss';
  import type { Snippet } from 'svelte';
  import { _ } from 'svelte-i18n';

  export interface TabItem {
    id:     string | number;
    label:  string;
    badge?: string | number;
  }

  interface Props {
    tabs:       TabItem[];
    activeTab?: string | number | null;
    onselect?:  (id: string | number) => void;
    onremove?:  (id: string | number) => void;
    actions?:   Snippet;
  }

  const { tabs, activeTab = null, onselect, onremove, actions } = $props<Props>();
</script>

<div class="tab-bar">
  <div class="tab-list">
    {#each tabs as tab (tab.id)}
      <div class="tab" class:active={activeTab === tab.id}>
        <button class="tab-btn" onclick={() => onselect?.(tab.id)}>
          {tab.label}
          {#if tab.badge !== undefined}
            <span class="tab-badge">({tab.badge})</span>
          {/if}
        </button>
        {#if onremove}
          <button class="tab-remove" title={$_('ui.remove')} onclick={() => onremove(tab.id)}>✕</button>
        {/if}
      </div>
    {/each}
  </div>

  {#if actions}
    <div class="tab-actions">
      {@render actions()}
    </div>
  {/if}
</div>
