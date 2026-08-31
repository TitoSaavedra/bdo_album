<script lang="ts">
  import './Titlebar.scss';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { _ } from 'svelte-i18n';
  import { settings } from '../../features/settings/state/settings.svelte';
  import SettingsPopover from '../../features/settings/components/SettingsPopover/SettingsPopover.svelte';

  const appWindow = getCurrentWindow();

  // Titlebar is mounted once, unconditionally, for the lifetime of the app —
  // the natural single place to keep <html>'s [data-theme]/[data-accent]
  // in sync with the persisted Settings state.
  $effect(() => {
    const root = document.documentElement;
    root.dataset.theme = settings.theme;
    if (settings.accent === 'amber') delete root.dataset.accent;
    else root.dataset.accent = settings.accent;
  });

  function minimize() {
    appWindow.minimize().catch(() => {});
  }
  function toggleMaximize() {
    appWindow.toggleMaximize().catch(() => {});
  }
  function close() {
    appWindow.close().catch(() => {});
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="titlebar-brand" data-tauri-drag-region>
    <span class="titlebar-mark">
      <svg viewBox="0 0 24 24" fill="none" width="62%" height="62%">
        <path d="M12 2.5 L19.5 8 L12 21.5 L4.5 8 Z" fill="currentColor" opacity=".18" />
        <path d="M12 2.5 L19.5 8 L12 21.5 L4.5 8 Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        <path d="M4.5 8 H19.5 M8.7 8 L12 2.5 M8.7 8 L12 21.5 M15.3 8 L12 2.5 M15.3 8 L12 21.5" stroke="currentColor" stroke-width=".9" opacity=".65" />
      </svg>
    </span>
    <span class="titlebar-name">Muse</span>
  </div>

  <div class="titlebar-controls">
    <SettingsPopover />
    <button class="win-btn" title={$_('titlebar.minimize')} onclick={minimize}>−</button>
    <button class="win-btn" title={$_('titlebar.maximize')} onclick={toggleMaximize}>▢</button>
    <button class="win-btn win-btn-close" title={$_('titlebar.close')} onclick={close}>×</button>
  </div>
</div>
