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
  }

  let {
    value = $bindable(''),
    options = [],
    onchange,
  }: Props = $props();

  function select(v: string | number) {
    value = v;
    onchange?.(v);
  }
</script>

<div class="pill-group">
  {#each options as opt}
    <div
      class="pill"
      class:pill-active={value === opt.value}
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
