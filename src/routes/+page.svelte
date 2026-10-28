<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { listen } from '@tauri-apps/api/event';
  import Minus from '@lucide/svelte/icons/minus';
  import Square from '@lucide/svelte/icons/square';
  import X from '@lucide/svelte/icons/x';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import Save from '@lucide/svelte/icons/save';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import { app } from '$lib/state/app.svelte';
  import { apply, connect, reset, togglePower, unsaved } from '$lib/state/board.svelte';
  import { loadConfig } from '$lib/state/config.svelte';
  import { openKbd } from '$lib/state/keyboard.svelte';
  import { nextProfile } from '$lib/state/profiles.svelte';
  import { loadSystem } from '$lib/state/system.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Profiles from '$lib/components/Profiles.svelte';
  import ProfileMenu from '$lib/components/ProfileMenu.svelte';
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

  let saving = $state(false);
  const changed = $derived(app.status === 'ok' && unsaved());

  async function save() {
    saving = true;
    await apply();
    saving = false;
  }

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
        <!-- правки видны на железе сразу; сохранение нужно, чтобы они пережили выключение ПК -->
        {#if changed}
          <button class="btn save" disabled={saving} title="Записать в память контроллера: иначе после выключения ПК вернётся сохранённое" onclick={save}><Save size={16} />Сохранить</button>
          <button class="icon-btn" title="Вернуть сохранённое в контроллере" onclick={reset}><RotateCcw size={18} /></button>
        {/if}
        <ProfileMenu />
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
      <Profiles />
    {:else if app.page === 'devices'}
      <DevicesPage />
    {:else}
      <SettingsPage />
    {/if}
  </main>
</div>

<style>
  .app { display: grid; grid-template-columns: auto 1fr; height: 100vh; }
  main { display: flex; flex-direction: column; gap: 14px; min-width: 0; min-height: 0; padding: 0 18px 16px 14px; }
  header { display: flex; align-items: center; gap: 16px; padding: 22px 0 6px 16px; }
  header > div:first-child { flex: 1; min-width: 0; }
  h1 { margin: 0; font-size: 22px; font-weight: 600; }
  header p { margin: 6px 0 0; color: var(--dim); }
  .tools { display: flex; gap: 10px; margin-top: 20px; }
  .save { display: flex; align-items: center; gap: 8px; padding: 0 14px; border-color: rgba(245, 181, 68, 0.5); background: rgba(245, 181, 68, 0.1); color: var(--warn); }
  .save:hover:not(:disabled) { background: rgba(245, 181, 68, 0.18); }
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
