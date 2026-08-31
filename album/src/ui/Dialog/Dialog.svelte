<script lang="ts">
  import './Dialog.scss';
  import { fade, scale } from 'svelte/transition';
  import Button from '../Button/Button.svelte';

  interface DialogInput { placeholder: string; }

  interface Props {
    title:       string;
    message?:    string;
    inputs?:     DialogInput[];
    error?:      string | null;
    submitting?: boolean;
    submitText?: string;
    cancelText?: string;
    onsubmit?:   (values: string[]) => void;
    oncancel?:   () => void;
  }

  let {
    title,
    message,
    inputs     = [],
    error      = null,
    submitting = false,
    submitText = 'Confirm',
    cancelText = 'Cancel',
    onsubmit,
    oncancel,
  }: Props = $props();

  let values = $state<string[]>([]);

  $effect(() => {
    values = inputs.map(() => '');
  });

  function focusFirst(node: HTMLElement, active: boolean) {
    if (active) node.focus();
  }

  function handleSubmit() {
    onsubmit?.(values);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') oncancel?.();
    if (e.key === 'Enter' && !e.shiftKey && inputs.length <= 1) handleSubmit();
  }

  const submitDisabled = $derived(
    submitting || (inputs.length > 0 && values.some(v => !v.trim()))
  );
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="dialog-backdrop"
  role="presentation"
  transition:fade={{ duration: 150 }}
  onclick={oncancel}
  onkeydown={handleKeydown}
>
  <div
    class="dialog"
    role="dialog"
    tabindex="-1"
    transition:scale={{ duration: 150, start: 0.95 }}
    onclick={e => e.stopPropagation()}
    onkeydown={e => e.stopPropagation()}
  >
    <h3>{title}</h3>

    {#if message}
      <p>{message}</p>
    {/if}

    {#if inputs.length > 0}
      <div class="inputs">
        {#each inputs as inp, i}
          <input
            type="text"
            placeholder={inp.placeholder}
            bind:value={values[i]}
            use:focusFirst={i === 0}
          />
        {/each}
      </div>
    {/if}

    {#if error}
      <div class="error">{error}</div>
    {/if}

    {#if submitting}
      <div class="loading-bar"><div class="loading-bar-fill"></div></div>
    {/if}

    <div class="actions">
      <Button variant="ghost" onclick={oncancel}>{cancelText}</Button>
      <Button variant="primary" disabled={submitDisabled} onclick={handleSubmit}>
        {submitting ? 'Loading...' : submitText}
      </Button>
    </div>
  </div>
</div>
