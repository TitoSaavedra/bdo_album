<script lang="ts">
  import './Sidebar.scss';
  import type { Snippet } from 'svelte';

  interface Props {
    children:  Snippet;
    width?:    number;
    side?:     'left' | 'right';
    open?:     boolean;
  }

  let {
    children,
    width = 220,
    side  = 'left',
    open  = $bindable(true),
  }: Props = $props();
</script>

<div class="sidebar-root" class:side-right={side === 'right'}>
  <aside
    class="sidebar-panel"
    class:collapsed={!open}
    style="--sidebar-w: {width}px"
  >
    {@render children()}
  </aside>

  <button
    class="sidebar-toggle"
    onclick={() => (open = !open)}
    title={open ? 'Collapse' : 'Expand'}
  >
    <svg
      width="10"
      height="10"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2.5"
      stroke-linecap="round"
      stroke-linejoin="round"
      style="transform: rotate({(side === 'left') === open ? '0deg' : '180deg'}); transition: transform 0.25s ease"
    >
      <polyline points="15 18 9 12 15 6"/>
    </svg>
  </button>
</div>
