<script lang="ts">
  import './Input.scss';

  interface Props {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    type?: 'text' | 'password' | 'number';
    icon?: import('svelte').Snippet;
    trailing?: import('svelte').Snippet;
    oninput?: (e: Event) => void;
    onchange?: (e: Event) => void;
    onkeydown?: (e: KeyboardEvent) => void;
  }

  let {
    value = $bindable(''),
    placeholder = '',
    disabled = false,
    type = 'text',
    icon,
    trailing,
    oninput,
    onchange,
    onkeydown,
  }: Props = $props();
</script>

<div class="input-wrap" class:has-icon={!!icon} class:has-trailing={!!trailing}>
  {#if icon}
    <span class="input-icon">{@render icon()}</span>
  {/if}
  <input
    {type}
    {placeholder}
    {disabled}
    bind:value
    class="field-input"
    {oninput}
    {onchange}
    {onkeydown}
  />
  {#if trailing}
    <span class="input-trailing">{@render trailing()}</span>
  {/if}
</div>
