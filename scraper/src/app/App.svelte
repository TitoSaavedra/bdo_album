<script lang="ts">
  import './App.scss';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { eventBus } from '$lib/events';
  import { getDbReady, getDbError, setDbReady, setClassIcons } from '../features/scraper/state/scraper.svelte';
  import Toast from '$ui/Toast/Toast.svelte';
  import type { ToastItem } from '$ui/Toast/Toast.svelte';
  import Dashboard from '../features/dashboard/Dashboard.svelte';
  import Logs      from '../features/logs/Logs.svelte';

  type Tab = 'dashboard' | 'logs';
  let tab = $state<Tab>('dashboard');

  const TABS: { id: Tab; label: string }[] = [
    { id: 'dashboard', label: 'Dashboard' },
    { id: 'logs',      label: 'Logs'      },
  ];

  const dbReady = $derived(getDbReady());
  const dbError = $derived(getDbError());

  const toasts = $derived<ToastItem[]>(
    dbReady === false
      ? [{ id: 1, type: 'error', text: `Database unavailable — ${dbError ?? 'check DATABASE_URL'}` }]
      : dbReady === null
        ? [{ id: 0, type: 'warning', text: 'Connecting to database...' }]
        : []
  );

  // ── Dock visibility ──
  let dockVisible = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | null = null;

  function onEnterDock() {
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
    dockVisible = true;
  }

  function onLeaveDock() {
    hideTimer = setTimeout(() => { dockVisible = false; }, 220);
  }

  onMount(async () => {
    await eventBus.init();
    // Poll until DB is ready — handles race between async DB init and frontend load
    const poll = async () => {
      const ready = await invoke<boolean>('get_db_status');
      if (ready) {
        setDbReady(true);
        const classes = await invoke<{ id: number; icon_svg: string | null }[]>('get_classes').catch(() => []);
        setClassIcons(classes);
      } else {
        setTimeout(poll, 500);
      }
    };
    poll();
  });
</script>

<div class="app">
  <main class="app-content">
    {#if tab === 'dashboard'}
      <Dashboard />
    {:else}
      <Logs />
    {/if}
  </main>

  <!-- Thin hover strip at the bottom that wakes the dock -->
  <div
    class="dock-trigger"
    onmouseenter={onEnterDock}
    onmouseleave={onLeaveDock}
    role="none"
  ></div>
</div>

<!-- Dock floats outside the flex layout so it can be fixed -->
<nav
  class="app-nav"
  class:visible={dockVisible}
  onmouseenter={onEnterDock}
  onmouseleave={onLeaveDock}
>
  {#each TABS as t}
    <button
      class="dock-item"
      class:active={tab === t.id}
      title={t.label}
      onclick={() => tab = t.id}
    >
      {#if t.id === 'dashboard'}
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="3" width="7" height="7" rx="1.5"/>
          <rect x="14" y="3" width="7" height="7" rx="1.5"/>
          <rect x="3" y="14" width="7" height="7" rx="1.5"/>
          <rect x="14" y="14" width="7" height="7" rx="1.5"/>
        </svg>
      {:else}
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="8" y1="13" x2="16" y2="13"/>
          <line x1="8" y1="17" x2="13" y2="17"/>
        </svg>
      {/if}
      {#if tab === t.id}
        <span class="dock-dot"></span>
      {/if}
    </button>
  {/each}
</nav>

<Toast {toasts} />
