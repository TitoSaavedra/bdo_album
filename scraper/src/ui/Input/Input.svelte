<script lang="ts">
  import './Input.scss';

  interface Props {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    type?: 'text' | 'password' | 'number';
    icon?: import('svelte').Snippet;
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
    oninput,
    onchange,
    onkeydown,
  }: Props = $props();
</script>

<div class="input-wrap" class:has-icon={!!icon}>
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
</div>
