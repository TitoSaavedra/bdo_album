<script lang="ts">
  import { Popover } from 'bits-ui';
  import { _ } from 'svelte-i18n';
  import { settings, setTheme, setAccent } from '../../state/settings.svelte';
  import type { AccentChoice } from '../../state/settings.svelte';

  const ACCENTS: { key: AccentChoice; swatch: string; labelKey: string }[] = [
    { key: 'amber',  swatch: '#f2a94e', labelKey: 'settings.accent_amber' },
    { key: 'violet', swatch: '#8b7bf0', labelKey: 'settings.accent_violet' },
    { key: 'coral',  swatch: '#f2735e', labelKey: 'settings.accent_coral' },
    { key: 'teal',   swatch: '#d8b98a', labelKey: 'settings.accent_teal' },
  ];
</script>

<Popover.Root>
  <Popover.Trigger title={$_('settings.title')} class="settings-trigger">
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="2.3" stroke="currentColor" stroke-width="1.4" />
      <path d="M8 1.6v1.7M8 12.7v1.7M14.4 8h-1.7M3.3 8H1.6M12.5 3.5l-1.2 1.2M4.7 11.3l-1.2 1.2M12.5 12.5l-1.2-1.2M4.7 4.7L3.5 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
    </svg>
  </Popover.Trigger>

  <Popover.Portal>
    <Popover.Content sideOffset={14} align="end" alignOffset={-4} class="settings-content">
      <div class="head">
        <span class="title">{$_('settings.title')}</span>
        <Popover.Close class="settings-close" aria-label={$_('ui.remove')}>✕</Popover.Close>
      </div>

      <div class="section">
        <div class="label">{$_('settings.appearance')}</div>
        <div class="seg">
          <button
            class="seg-btn"
            class:active={settings.theme === 'light'}
            onclick={() => setTheme('light')}
          >{$_('settings.light')}</button>
          <button
            class="seg-btn"
            class:active={settings.theme === 'dark'}
            onclick={() => setTheme('dark')}
          >{$_('settings.dark')}</button>
        </div>
      </div>

      <div class="section">
        <div class="label">{$_('settings.accent_color')}</div>
        <div class="swatches">
          {#each ACCENTS as a (a.key)}
            <button
              class="swatch"
              title={$_(a.labelKey)}
              aria-label={$_(a.labelKey)}
              style="background:{a.swatch}; box-shadow:{settings.accent === a.key
                ? '0 0 0 2px var(--color-modal-bg), 0 0 0 4px var(--color-text-primary)'
                : '0 0 0 1px var(--color-border)'};"
              onclick={() => setAccent(a.key)}
            ></button>
          {/each}
        </div>
      </div>
    </Popover.Content>
  </Popover.Portal>
</Popover.Root>

<style lang="scss">
  @use './SettingsPopover.scss';
</style>
