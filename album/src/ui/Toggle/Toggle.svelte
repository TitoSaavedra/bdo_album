<script lang="ts">
  import './Toggle.scss';

  interface Props {
    checked?: boolean;
    onclick?: (e: MouseEvent) => void;
  }

  let {
    checked = $bindable(false),
    onclick,
  }: Props = $props();
</script>

<div
  class="toggle"
  class:toggle-on={checked}
  role="switch"
  aria-checked={checked}
  tabindex="0"
  {onclick}
  onkeydown={(e) => (e.key === ' ' || e.key === 'Enter') && onclick?.(e as unknown as MouseEvent)}
>
  <span class="knob"></span>
</div>
