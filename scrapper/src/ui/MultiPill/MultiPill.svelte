<script lang="ts">
  import './MultiPill.scss';

  interface Props {
    options:    string[];
    selected:   string[];
    onchange:   (selected: string[]) => void;
    disabled?:  boolean;
    reset?:     boolean;
    selectAll?: boolean;
    icons?:     Record<string, string>;
  }

  let { options, selected, onchange, disabled = false, reset, selectAll, icons }: Props = $props();

  function toggle(v: string) {
    if (disabled) return;
    const next = selected.includes(v)
      ? selected.filter(s => s !== v)
      : [...selected, v];
    if (next.length > 0) onchange(next);
  }

  function doReset() {
    if (disabled) return;
    onchange([options[0]]);
  }

  function doSelectAll() {
    if (disabled) return;
    onchange([...options]);
  }
</script>

<div class="mp-wrap" class:mp-disabled={disabled}>
  {#each options as opt}
    <button
      class="mp-pill"
      class:mp-active={selected.includes(opt)}
      onclick={() => toggle(opt)}
    >
      {#if icons?.[opt]}
        <span class="mp-icon">{@html icons[opt]}</span>
      {/if}
      {opt}
    </button>
  {/each}
  {#if selectAll}
    <button
      class="mp-reset"
      class:mp-active={selected.length === options.length}
      onclick={doSelectAll}
      title="Select all"
      disabled={disabled}
    >↻</button>
  {/if}
  {#if reset}
    <button class="mp-reset" onclick={doReset} title="Reset" disabled={disabled}>↺</button>
  {/if}
</div>
