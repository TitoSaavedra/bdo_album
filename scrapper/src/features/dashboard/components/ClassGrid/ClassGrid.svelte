<script lang="ts">
  import './ClassGrid.scss';
  import { getClassMap, getClassIcons } from '../../../scrapper/state/scrapper.svelte';

  const classMap   = $derived(getClassMap());
  const classIcons = $derived(getClassIcons());
  const classes    = $derived(Object.values(classMap));
</script>

<div class="class-grid">
  {#each classes as cls (cls.id)}
    <div class="class-card" class:active={cls.active}>
      <div class="class-icon">
        {#if classIcons[cls.id]}
          {@html classIcons[cls.id]}
        {:else}
          {cls.name[0]}
        {/if}
      </div>
      <span class="class-name">{cls.name}</span>
      <div class="class-stats">
        <span class="class-fetched">{cls.fetched}</span>
        {#if cls.images > 0}
          <span class="class-images">↑{cls.images}</span>
        {/if}
      </div>
      {#if cls.active}
        <div class="class-pulse"></div>
      {/if}
    </div>
  {/each}
</div>
