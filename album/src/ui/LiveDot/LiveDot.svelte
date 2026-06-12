<script lang="ts">
  import { beauty } from '../../features/beauty/state/beauty.svelte';
</script>

{#if beauty.dbReady}
  <div
    class="live-dot"
    class:connected={beauty.listenerConnected}
    title={beauty.listenerConnected ? 'Scraper listener connected' : 'Scraper listener disconnected'}
  ></div>
{/if}

<style lang="scss">
  .live-dot {
    position: fixed;
    bottom: 14px;
    right: 14px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #c0392b;
    box-shadow: 0 0 0 0 rgba(192, 57, 43, 0.5);
    animation: pulse-red 2s ease-in-out infinite;
    z-index: 9999;
    pointer-events: none;
  }

  .live-dot.connected {
    background: #3d9e4a;
    box-shadow: 0 0 0 0 rgba(61, 158, 74, 0.5);
    animation: pulse-green 2s ease-in-out infinite;
    pointer-events: auto;
  }

  @keyframes pulse-red {
    0%, 100% { box-shadow: 0 0 0 0 rgba(192, 57, 43, 0.5); }
    50%       { box-shadow: 0 0 0 5px rgba(192, 57, 43, 0); }
  }

  @keyframes pulse-green {
    0%, 100% { box-shadow: 0 0 0 0 rgba(61, 158, 74, 0.5); }
    50%       { box-shadow: 0 0 0 5px rgba(61, 158, 74, 0); }
  }
</style>
