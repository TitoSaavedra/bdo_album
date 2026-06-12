<script lang="ts">
  import './PillSelector.scss';

  interface Option {
    value: string | number;
    label: string;
    icon?: string;
    color?: string;
  }

  interface Props {
    value?: string | number;
    options?: Option[];
    onchange?: (value: string | number) => void;
    disabled?: boolean;
  }

  let {
    value = '',
    options = [],
    onchange,
    disabled = false,
  }: Props = $props();

  let selected = $state<string | number>('');

  $effect(() => { selected = value; });

  function select(v: string | number) {
    if (disabled) return;
    selected = v;
    onchange?.(v);
  }
</script>

<div class="pill-group" class:pill-group-disabled={disabled}>
  {#each options as opt}
    <div
      class="pill"
      class:pill-active={selected === opt.value}
      style={opt.color ? `--pill-color: ${opt.color}` : ''}
      role="button"
      tabindex="0"
      onclick={() => select(opt.value)}
      onkeydown={(e) => e.key === 'Enter' && select(opt.value)}
    >
      {#if opt.icon}<span>{opt.icon}</span>{/if}
      {opt.label}
    </div>
  {/each}
</div>
