<script lang="ts">
  import './Toast.scss';

  export interface ToastItem {
    id: number;
    type: 'success' | 'warning' | 'error';
    text: string;
    onClick?: () => void;
  }

  const ICONS = { success: '✓', warning: '⚠', error: '✕' };

  interface Props {
    toasts?: ToastItem[];
    ondismiss?: (id: number) => void;
  }

  let { toasts = [], ondismiss }: Props = $props();
</script>

<div class="toast-container">
  {#each toasts as t (t.id)}
    <div
      class="toast toast-{t.type}"
      class:clickable={!!t.onClick}
      role="alert"
      onclick={() => { t.onClick?.(); ondismiss?.(t.id); }}
    >
      <span class="toast-icon">{ICONS[t.type]}</span>
      <span class="toast-text">{t.text}</span>
    </div>
  {/each}
</div>
