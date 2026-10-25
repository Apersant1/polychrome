<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Minus from '@lucide/svelte/icons/minus';
  import Square from '@lucide/svelte/icons/square';
  import X from '@lucide/svelte/icons/x';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import { app, apply, connect, loadProfiles, loadSystem, unsaved, useProfile } from '$lib/app.svelte';
  import Sidebar from '$lib/Sidebar.svelte';
  import DeviceTree from '$lib/DeviceTree.svelte';
  import Preview from '$lib/Preview.svelte';
  import ColorPanel from '$lib/ColorPanel.svelte';
  import Effects from '$lib/Effects.svelte';
  import ApplyPanel from '$lib/ApplyPanel.svelte';
  import Profiles from '$lib/Profiles.svelte';
  import DevicesPage from '$lib/DevicesPage.svelte';
  import SettingsPage from '$lib/SettingsPage.svelte';

  const win = getCurrentWindow();
  const TITLES = {
    home: ['Управление подсветкой', 'Настройте цвет, режимы и эффекты для ваших устройств'],
    devices: ['Устройства', 'Зоны контроллера платы и остальное железо компьютера'],
    effects: ['Эффекты', 'Все режимы контроллера с живым превью'],
    profiles: ['Профили', 'Наборы настроек всех зон, переключаются одним кликом'],
    settings: ['Настройки', 'Контроллер и сохранение'],
  };

  onMount(() => {
    loadProfiles();
    connect();
    loadSystem();
    let frame = requestAnimationFrame(function tick(ms) {
      app.now = ms / 1000;
      frame = requestAnimationFrame(tick);
    });
    // Без записи в контроллер подсветка откатится после перезагрузки — спрашиваем перед выходом
    const unlisten = win.onCloseRequested((e) => {
      if (app.status !== 'ok' || !unsaved()) return;
      e.preventDefault();
      app.closing = true;
    });
    return () => {
      cancelAnimationFrame(frame);
      unlisten.then((f) => f());
    };
  });

  async function saveAndQuit() {
    await apply();
    if (!unsaved()) win.destroy();
    else app.closing = false; // запись не удалась — ошибка видна в баннере
  }
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
        <button aria-label="Закрыть" class="close" onclick={() => win.close()}><X size={17} /></button>
      </div>
    </header>

    {#if app.status === 'error'}
      <div class="banner">
        {app.error}
        <button class="btn" onclick={connect}>Повторить</button>
      </div>
    {/if}

    {#if app.page === 'home'}
      <div class="home">
        <div class="tree"><DeviceTree /></div>
        <div class="view"><Preview /></div>
        <div class="color"><ColorPanel /></div>
        <div class="fx"><Effects /></div>
        <div class="targets"><ApplyPanel /></div>
        <div class="prof"><Profiles /></div>
      </div>
    {:else if app.page === 'effects'}
      <div class="split">
        <Effects all />
        <ColorPanel />
      </div>
    {:else if app.page === 'profiles'}
      <Profiles grid />
    {:else if app.page === 'devices'}
      <DevicesPage />
    {:else}
      <SettingsPage />
    {/if}
  </main>
</div>

{#if app.closing}
  <div class="scrim">
    <div class="dialog panel" role="alertdialog" aria-modal="true" aria-labelledby="quit-title">
      <h3 id="quit-title">Записать изменения в контроллер?</h3>
      <p>Подсветка уже горит по-новому, но без записи после перезагрузки компьютера она вернётся к прежней.</p>
      <div class="actions">
        <button class="btn" onclick={() => (app.closing = false)}>Отмена</button>
        <button class="btn" onclick={() => win.destroy()}>Выйти без записи</button>
        <!-- svelte-ignore a11y_autofocus -->
        <button class="btn primary" autofocus onclick={saveAndQuit}>Записать и выйти</button>
      </div>
    </div>
  </div>
{/if}

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
  .home {
    flex: 1;
    display: grid;
    grid-template-columns: 250px minmax(0, 1fr) 330px;
    grid-template-rows: minmax(0, 1.25fr) minmax(0, 1fr) auto;
    grid-template-areas: 'tree view color' 'tree fx targets' 'prof prof prof';
    gap: 14px;
    min-height: 0;
  }
  .home > div { display: grid; min-height: 0; min-width: 0; }
  .tree { grid-area: tree; }
  .view { grid-area: view; }
  .color { grid-area: color; }
  .fx { grid-area: fx; }
  .targets { grid-area: targets; }
  .prof { grid-area: prof; }
  .scrim { position: fixed; inset: 0; display: grid; place-items: center; background: rgba(0, 0, 0, 0.55); z-index: 20; }
  .dialog { max-width: 440px; }
  .dialog p { margin: 0 0 18px; color: var(--dim); line-height: 1.5; }
  .actions { display: flex; justify-content: flex-end; gap: 10px; }
  .actions .btn { padding: 10px 14px; }
  .split { flex: 1; display: grid; grid-template-columns: minmax(0, 1fr) 350px; gap: 14px; min-height: 0; align-items: start; }
</style>
