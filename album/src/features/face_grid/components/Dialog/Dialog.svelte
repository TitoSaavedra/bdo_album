<script lang="ts">
  import { Dialog } from '../../../../ui';
  import { closeDialog, faceGrid } from '../../state/face_grid.svelte';

  const d = $derived(faceGrid.dialog);

  const inputs = $derived(
    (d.inputs ?? []).map(i => ({ placeholder: i.placeholder }))
  );

  async function handleSubmit(values: string[]) {
    await d.onSubmit?.(values);
  }
</script>

{#if d.open}
  <Dialog
    title={d.title}
    message={d.message ?? undefined}
    {inputs}
    error={d.error ?? undefined}
    submitting={d.submitting}
    submitText={d.submitText || 'Confirm'}
    onsubmit={handleSubmit}
    oncancel={closeDialog}
  />
{/if}
