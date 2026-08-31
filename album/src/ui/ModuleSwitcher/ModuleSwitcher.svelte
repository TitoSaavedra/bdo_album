<script lang="ts">
  import './ModuleSwitcher.scss';
  import { _ } from 'svelte-i18n';

  type Module = 'beauty' | 'face_grid';

  interface Props {
    active:   Module;
    onchange: (tab: Module) => void;
  }

  const { active, onchange }: Props = $props();
</script>

<div class="module-switcher">
  <button class="mod-btn" class:active={active === 'beauty'} onclick={() => onchange('beauty')}>
    {$_('app.tabs.beauty')}
  </button>
  <button class="mod-btn" class:active={active === 'face_grid'} onclick={() => onchange('face_grid')}>
    {$_('app.tabs.face_grid')}
  </button>
</div>
