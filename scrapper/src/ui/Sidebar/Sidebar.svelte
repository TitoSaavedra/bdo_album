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
    {side === 'left' ? (open ? '‹' : '›') : (open ? '›' : '‹')}
  </button>
</div>
