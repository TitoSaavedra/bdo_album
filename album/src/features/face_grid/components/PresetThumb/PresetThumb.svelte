<script lang="ts">
  import { onMount } from 'svelte';

  let { urls }: { urls: string[] } = $props();

  const MAX = 4;
  let display = $derived(urls.filter(u => u).slice(0, MAX));

  let current = $state(0);
  let visible = $state(true);

  onMount(() => {
    if (display.length <= 1) return;
    const id = setInterval(() => {
      visible = false;
      setTimeout(() => {
        current = (current + 1) % display.length;
        visible = true;
      }, 300);
    }, 2000);
    return () => clearInterval(id);
  });
</script>

<div class="preset-thumb">
  {#if display[current]}
    <img
      src={display[current]}
      alt=""
      class="thumb-img"
      class:fade-out={!visible}
    />
  {:else}
    <div class="thumb-empty"></div>
  {/if}

  {#if display.length > 1}
    <div class="thumb-dots">
      {#each display as _, i}
        <span class="dot" class:active={i === current}></span>
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
  @use './PresetThumb.scss';
</style>
