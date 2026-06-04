<script lang="ts">
  import './App.scss';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { eventBus } from '$lib/events';
  import { getDbReady, getDbError, setDbReady, setClassIcons } from '../features/scrapper/state/scrapper.svelte';
  import Toast from '$ui/Toast/Toast.svelte';
  import type { ToastItem } from '$ui/Toast/Toast.svelte';
  import Dashboard from '../features/dashboard/Dashboard.svelte';
  import Logs      from '../features/logs/Logs.svelte';

  type Tab = 'dashboard' | 'logs';
  let tab = $state<Tab>('dashboard');

  const TABS: { id: Tab; icon: string; label: string }[] = [
    { id: 'dashboard', icon: '⚡', label: 'Dashboard' },
    { id: 'logs',      icon: '📋', label: 'Logs'      },
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

  <nav class="app-nav">
    <div class="nav-brand">BDO</div>
    {#each TABS as t}
      <button
        class="nav-item"
        class:active={tab === t.id}
        title={t.label}
        onclick={() => tab = t.id}
      >
        <span class="nav-icon">{t.icon}</span>
        <span class="nav-label">{t.label}</span>
      </button>
    {/each}
  </nav>

  <main class="app-content">
    {#if tab === 'dashboard'}
      <Dashboard />
    {:else}
      <Logs />
    {/if}
  </main>

</div>

<Toast {toasts} />
