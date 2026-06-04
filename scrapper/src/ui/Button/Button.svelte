<script lang="ts">
  import './Button.scss';
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'ghost' | 'icon';
    disabled?: boolean;
    active?: boolean;
    type?: 'button' | 'submit' | 'reset';
    title?: string;
    class?: string;
    onclick?: (e: MouseEvent) => void;
    children?: Snippet;
  }

  let {
    variant = 'primary',
    disabled = false,
    active = false,
    type = 'button',
    title = '',
    class: className = '',
    onclick,
    children,
  }: Props = $props();
</script>

<button
  {type}
  {disabled}
  {title}
  {onclick}
  class="btn btn-{variant} {className}"
  class:active
>
  {@render children?.()}
</button>
