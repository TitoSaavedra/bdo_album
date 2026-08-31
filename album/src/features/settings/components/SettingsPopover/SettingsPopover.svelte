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
  <Popover.Trigger
    title={$_('settings.title')}
    class="grid h-8 w-8 flex-shrink-0 place-items-center rounded-full border transition-colors"
    style="background:var(--color-bg-elevated); border-color:var(--color-border); color:var(--color-text-secondary);"
  >
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="2.3" stroke="currentColor" stroke-width="1.4" />
      <path d="M8 1.6v1.7M8 12.7v1.7M14.4 8h-1.7M3.3 8H1.6M12.5 3.5l-1.2 1.2M4.7 11.3l-1.2 1.2M12.5 12.5l-1.2-1.2M4.7 4.7L3.5 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
    </svg>
  </Popover.Trigger>

  <Popover.Portal>
    <Popover.Content
      sideOffset={8}
      align="end"
      class="z-50 w-64 rounded-2xl border p-4 shadow-2xl"
      style="background:var(--color-bg-elevated); border-color:var(--color-modal-border);"
    >
      <div class="mb-3 text-[11.5px] font-bold" style="color:var(--color-text-primary)">
        {$_('settings.title')}
      </div>

      <div class="mb-3.5">
        <div class="mb-1.5 text-[9.5px] font-bold uppercase tracking-widest" style="color:var(--color-text-muted)">
          {$_('settings.appearance')}
        </div>
        <div class="flex gap-1 rounded-lg border p-1" style="background:var(--color-bg-base); border-color:var(--color-border);">
          <button
            class="flex-1 rounded-md py-1.5 text-center text-[11px] font-bold"
            style={settings.theme === 'light' ? 'background:var(--color-bg-card); color:var(--color-accent);' : 'color:var(--color-text-secondary);'}
            onclick={() => setTheme('light')}
          >{$_('settings.light')}</button>
          <button
            class="flex-1 rounded-md py-1.5 text-center text-[11px] font-bold"
            style={settings.theme === 'dark' ? 'background:var(--color-bg-card); color:var(--color-accent);' : 'color:var(--color-text-secondary);'}
            onclick={() => setTheme('dark')}
          >{$_('settings.dark')}</button>
        </div>
      </div>

      <div>
        <div class="mb-1.5 text-[9.5px] font-bold uppercase tracking-widest" style="color:var(--color-text-muted)">
          {$_('settings.accent_color')}
        </div>
        <div class="flex gap-2">
          {#each ACCENTS as a (a.key)}
            <button
              title={$_(a.labelKey)}
              aria-label={$_(a.labelKey)}
              class="h-7 w-7 rounded-full"
              style="background:{a.swatch}; box-shadow:{settings.accent === a.key
                ? '0 0 0 2px var(--color-bg-elevated), 0 0 0 3.5px var(--color-text-primary)'
                : '0 0 0 1px var(--color-border)'};"
              onclick={() => setAccent(a.key)}
            ></button>
          {/each}
        </div>
      </div>
    </Popover.Content>
  </Popover.Portal>
</Popover.Root>
