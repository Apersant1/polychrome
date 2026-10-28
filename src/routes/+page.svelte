<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { listen } from '@tauri-apps/api/event';
  import Minus from '@lucide/svelte/icons/minus';
  import Square from '@lucide/svelte/icons/square';
  import X from '@lucide/svelte/icons/x';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import { app } from '$lib/state/app.svelte';
  import { connect, togglePower } from '$lib/state/board.svelte';
  import { loadConfig } from '$lib/state/config.svelte';
  import { openKbd } from '$lib/state/keyboard.svelte';
  import { nextProfile, useProfile } from '$lib/state/profiles.svelte';
  import { loadSystem } from '$lib/state/system.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Profiles from '$lib/components/Profiles.svelte';
  import HomePage from '$lib/pages/HomePage.svelte';
  import EffectsPage from '$lib/pages/EffectsPage.svelte';
  import DevicesPage from '$lib/pages/DevicesPage.svelte';
  import SettingsPage from '$lib/pages/SettingsPage.svelte';

  const win = getCurrentWindow();
  const TITLES = {
    home: ['Управление подсветкой', 'Настройте цвет, режимы и эффекты для ваших устройств'],
    devices: ['Устройства', 'Зоны контроллера платы и остальное железо компьютера'],
    effects: ['Эффекты', 'Все режимы контроллера с живым превью'],
    profiles: ['Профили', 'Наборы настроек всех зон, переключаются одним кликом'],
    settings: ['Настройки', 'Контроллер и сохранение'],
  };

  onMount(() => {
    loadConfig().then(openKbd); // режим клавиатуры, который считать включённым, лежит в конфиге
    connect();
    loadSystem();
    let frame = requestAnimationFrame(function tick(ms) {
      app.now = ms / 1000;
      frame = requestAnimationFrame(tick);
    });
    const unhotkey = listen<string>('hotkey', (e) => (e.payload === 'toggle' ? togglePower() : nextProfile()));
    return () => {
      cancelAnimationFrame(frame);
      unhotkey.then((f) => f());
    };
  });
</script>

<div class="app">
  <Sidebar />
  <main>
    <header data-tauri-drag-region>
      <div data-tauri-drag-region>
        <h1 data-tauri-drag-region>{TITLES[app.page][0]}</h1>
        <p data-tauri-drag-region>{TITLES[app.page][1]}</p>
      </div>
      <div class="tools">
        <select aria-label="Профиль" value={app.profile} onchange={(e) => useProfile(e.currentTarget.value)} disabled={app.status !== 'ok'}>
          <option value="" disabled>Профиль…</option>
          {#each app.profiles as p (p.name)}<option value={p.name}>{p.name}</option>{/each}
        </select>
        <button class="icon-btn" title="Переподключить контроллер" onclick={connect}><RefreshCw size={18} /></button>
      </div>
      <div class="window">
        <button aria-label="Свернуть" onclick={() => win.minimize()}><Minus size={16} /></button>
        <button aria-label="Развернуть" onclick={() => win.toggleMaximize()}><Square size={13} /></button>
        <button aria-label="Свернуть в трей" title="Свернуть в трей" class="close" onclick={() => win.close()}><X size={17} /></button>
      </div>
    </header>

    {#if app.status === 'error'}
      <div class="banner">
        {app.error}
        <button class="btn" onclick={connect}>Повторить</button>
      </div>
    {/if}

    {#if app.page === 'home'}
      <HomePage />
    {:else if app.page === 'effects'}
      <EffectsPage />
    {:else if app.page === 'profiles'}
      <Profiles grid />
    {:else if app.page === 'devices'}
      <DevicesPage />
    {:else}
      <SettingsPage />
    {/if}
  </main>
</div>

<style>
  .app { display: grid; grid-template-columns: 270px 1fr; height: 100vh; }
  main { display: flex; flex-direction: column; gap: 14px; min-width: 0; min-height: 0; padding: 0 18px 16px 14px; }
  header { display: flex; align-items: center; gap: 16px; padding: 22px 0 6px 16px; }
  header > div:first-child { flex: 1; min-width: 0; }
  h1 { margin: 0; font-size: 22px; font-weight: 600; }
  header p { margin: 6px 0 0; color: var(--dim); }
  .tools { display: flex; gap: 10px; margin-top: 20px; }
  .tools select { min-width: 180px; }
  .window { position: fixed; top: 0; right: 0; display: flex; }
  .window button { display: grid; place-items: center; width: 46px; height: 32px; border: 0; background: none; color: #c4ccda; }
  .window button:hover { background: rgba(255, 255, 255, 0.08); }
  .window .close:hover { background: #e81123; color: #fff; }
  .banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 16px;
    border-radius: 10px;
    border: 1px solid rgba(255, 107, 107, 0.4);
    background: rgba(255, 107, 107, 0.08);
  }
</style>
