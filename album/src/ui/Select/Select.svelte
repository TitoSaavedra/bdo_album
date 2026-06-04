<script lang="ts">
  import './Select.scss';

  interface Option {
    value: string | number;
    label: string;
  }

  interface Props {
    value?: string | number;
    options?: Option[];
    onchange?: (e: Event) => void;
  }

  let {
    value = $bindable(''),
    options = [],
    onchange,
  }: Props = $props();
</script>

<select class="field-select" bind:value {onchange}>
  {#each options as opt}
    <option value={opt.value}>{opt.label}</option>
  {/each}
</select>
