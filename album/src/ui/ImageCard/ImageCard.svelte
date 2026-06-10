<script lang="ts">
  import './ImageCard.scss';

  interface Props {
    src?:    string | null;
    badge?:  string | number;
    ondrop?: (e: DragEvent) => void;
  }

  let { src, badge, ondrop }: Props = $props();

  let dragOver = $state(false);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="image-card"
  class:drag-over={dragOver}
  ondragover={(e) => { e.preventDefault(); dragOver = true; }}
  ondragleave={() => (dragOver = false)}
  ondrop={(e) => { e.preventDefault(); dragOver = false; ondrop?.(e); }}
>
  {#if src}
    <img {src} alt="" class="card-img" />
  {/if}
  <div class="card-overlay"></div>
  {#if badge !== undefined}
    <span class="card-badge">{badge}</span>
  {/if}
</div>
