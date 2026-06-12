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

<div class="toggle" class:toggle-on={checked} role="switch" aria-checked={checked} {onclick}>
  <span class="knob"></span>
</div>
