<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { Dialog } from '../../../../ui';
  import { getDialog, closeDialog } from '../../state/face_grid.svelte';

  const d = $derived(getDialog());

  async function handleSubmit(values: string[]) {
    await d.onSubmit?.(values);
  }
</script>

{#if d.open}
  <Dialog
    title={d.title}
    message={d.message ?? undefined}
    inputs={(d.inputs ?? []).map(i => ({ placeholder: i.placeholder }))}
    error={d.error ?? undefined}
    submitting={d.submitting}
    submitText={d.submitText || $_('face_grid.dialog.confirm')}
    onsubmit={handleSubmit}
    oncancel={closeDialog}
  />
{/if}
