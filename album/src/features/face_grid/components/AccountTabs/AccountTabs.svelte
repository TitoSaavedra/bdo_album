<script lang="ts">
  import './AccountTabs.scss';
  import TabBar from '../../../../ui/TabBar/TabBar.svelte';
  import type { TabItem } from '../../../../ui/TabBar/TabBar.svelte';
  import type { BdoAccount } from '../../../../lib/face_grid';
  import { faceGrid, removeAccount, selectAccount } from '../../state/face_grid.svelte';

  interface Props {
    accounts: BdoAccount[];
    onSave?:  () => void;
    onReset?: () => void;
  }
  const { accounts, onSave, onReset }: Props = $props();

  const tabs = $derived(
    accounts.map(acc => ({
      id:    acc.account_id,
      label: `Account ${acc.account_id}`,
      badge: acc.characters.length,
    } satisfies TabItem))
  );
</script>

{#snippet tabActions()}
  <button class="btn-reset" onclick={onReset} title="Reset">
    <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
      <path d="M1 4v6h6M23 20v-6h-6M20.3 5a9 9 0 0 0-15.2 1.5M3.7 19a9 9 0 0 0 15.2-1.5"
            stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>
  <button class="btn-save" onclick={onSave}>Save Preset</button>
{/snippet}

<TabBar
  tabs={tabs}
  activeTab={faceGrid.activeAccountId}
  onselect={id => selectAccount(String(id))}
  onremove={id => removeAccount(String(id))}
  actions={tabActions}
/>
