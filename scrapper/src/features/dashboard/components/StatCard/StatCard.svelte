<script lang="ts">
  import './StatCard.scss';

  interface Props {
    label: string;
    value: string | number;
    sub?:  string;
    color?: string;
    icon?: string;
  }

  let { label, value, sub, color, icon }: Props = $props();
</script>

<div class="stat-card" style={color ? `--card-color: ${color}` : ''}>
  <div class="stat-header">
    <span class="stat-label">{label}</span>
    {#if icon}<span class="stat-icon">{icon}</span>{/if}
  </div>
  <span class="stat-value">{value}</span>
  {#if sub}<span class="stat-sub">{sub}</span>{/if}
</div>
