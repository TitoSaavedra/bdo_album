<script lang="ts">
  import './SidebarList.scss';

  export interface SidebarItem {
    id:       string;
    title:    string;
    subtitle?: string;
  }

  export interface ItemAction {
    label:    string;
    variant?: 'primary' | 'danger';
    onclick:  (id: string) => void;
  }

  interface Props {
    heading?:   string;
    items:      SidebarItem[];
    actions?:   ItemAction[];
    busy?:      boolean;
    emptyText?: string;
  }

  const {
    heading,
    items,
    actions   = [],
    busy      = false,
    emptyText = 'No items',
  }: Props = $props();
</script>

<aside class="sidebar-list">
  {#if heading}
    <div class="sidebar-heading">{heading}</div>
  {/if}

  <div class="sidebar-items custom-scroll">
    {#if items.length === 0}
      <div class="sidebar-empty">{emptyText}</div>
    {:else}
      {#each items as item (item.id)}
        <div class="sidebar-item">
          <div class="item-info">
            <div class="item-title">{item.title}</div>
            {#if item.subtitle}
              <div class="item-subtitle">{item.subtitle}</div>
            {/if}
          </div>

          {#if actions.length > 0}
            <div class="item-actions">
              {#each actions as action}
                <button
                  class="item-btn item-btn--{action.variant ?? 'primary'}"
                  disabled={busy}
                  onclick={() => action.onclick(item.id)}
                >
                  {action.label}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</aside>

