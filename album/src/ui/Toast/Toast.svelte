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
    {#if t.onClick}
      <button
        class="toast toast-{t.type} clickable"
        onclick={() => { t.onClick?.(); ondismiss?.(t.id); }}
      >
        <span class="toast-icon">{ICONS[t.type]}</span>
        <span class="toast-text">{t.text}</span>
      </button>
    {:else}
      <div class="toast toast-{t.type}" role="alert">
        <span class="toast-icon">{ICONS[t.type]}</span>
        <span class="toast-text">{t.text}</span>
      </div>
    {/if}
  {/each}
</div>
