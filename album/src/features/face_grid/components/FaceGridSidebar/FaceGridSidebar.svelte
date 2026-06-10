<script lang="ts">
  import SidebarList from '../../../../ui/SidebarList/SidebarList.svelte';
  import type { SidebarItem, ItemAction } from '../../../../ui/SidebarList/SidebarList.svelte';
  import { applyGrid, deleteGrid, faceGrid, openConfirmDialog } from '../../state/face_grid.svelte';

  const items = $derived(
    faceGrid.savedGrids.map(g => ({
      id:       g.id,
      title:    g.name,
      subtitle: `Account ${g.account_id}`,
    } satisfies SidebarItem))
  );

  const actions: ItemAction[] = [
    {
      label:   'Apply',
      variant: 'primary',
      onclick: (id) => openConfirmDialog(
        'Apply Grid',
        'FaceTexture images will be overwritten. Continue?',
        () => applyGrid(id),
        'Apply'
      ),
    },
    {
      label:   'Delete',
      variant: 'danger',
      onclick: (id) => openConfirmDialog(
        'Delete Grid',
        'Are you sure? This action cannot be undone.',
        () => deleteGrid(id),
        'Delete'
      ),
    },
  ];
</script>

<SidebarList
  heading="Saved Grids"
  {items}
  {actions}
  busy={faceGrid.applyingGrid}
  emptyText="No saved grids"
/>
