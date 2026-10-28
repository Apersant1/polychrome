<script lang="ts">
  import House from '@lucide/svelte/icons/house';
  import CircuitBoard from '@lucide/svelte/icons/circuit-board';
  import Sparkles from '@lucide/svelte/icons/sparkles';
  import User from '@lucide/svelte/icons/user';
  import Settings from '@lucide/svelte/icons/settings';
  import PanelLeftClose from '@lucide/svelte/icons/panel-left-close';
  import PanelLeftOpen from '@lucide/svelte/icons/panel-left-open';
  import { app, type Page } from '$lib/state/app.svelte';

  const NAV: [Page, string, typeof House][] = [
    ['home', 'Главная', House],
    ['devices', 'Устройства', CircuitBoard],
    ['effects', 'Эффекты', Sparkles],
    ['profiles', 'Профили', User],
    ['settings', 'Настройки', Settings],
  ];

  // Свёрнута ли панель до иконок: настройка этого компьютера, хватает localStorage
  const KEY = 'polychrome.sidebar';
  let collapsed = $state(load());

  function load() {
    try {
      return localStorage.getItem(KEY) === 'collapsed';
    } catch {
      return false;
    }
  }

  function toggle() {
    collapsed = !collapsed;
    try {
      localStorage.setItem(KEY, collapsed ? 'collapsed' : '');
    } catch {
      // не критично: при следующем запуске панель просто будет развёрнута
    }
  }

  const status = $derived(
    app.status === 'ok' ? 'Контроллер подключён' : app.status === 'connecting' ? 'Подключение…' : 'Нет связи с контроллером',
  );
</script>

<aside class:collapsed>
  <header data-tauri-drag-region>
    <svg width="34" height="30" viewBox="0 0 34 30" aria-hidden="true">
      <defs>
        <linearGradient id="logo" x1="0" y1="1" x2="1" y2="0">
          <stop offset="0" stop-color="#ff3d6e" /><stop offset="0.5" stop-color="#a64dff" /><stop offset="1" stop-color="#2f9bff" />
        </linearGradient>
      </defs>
      <path d="M17 2 32 28H2Z" fill="none" stroke="url(#logo)" stroke-width="4" stroke-linejoin="round" />
    </svg>
    <b class="text">Polychrome</b>
    <small class="text">v0.1.0</small>
  </header>

  <nav>
    {#each NAV as [page, label, Icon] (page)}
      <button class:active={app.page === page} title={collapsed ? label : undefined} onclick={() => (app.page = page)}>
        <Icon size={20} /><span class="text">{label}</span>
      </button>
    {/each}
  </nav>

  <footer>
    <button class="fold" title={collapsed ? 'Развернуть панель' : undefined} aria-expanded={!collapsed} onclick={toggle}>
      {#if collapsed}<PanelLeftOpen size={20} />{:else}<PanelLeftClose size={20} /><span class="text">Свернуть</span>{/if}
    </button>
    <p class="status" title={collapsed ? status : undefined}><i class={app.status}></i><span class="text">{status}</span></p>
    <ul class="text">
      <li><i class={app.status}></i>ASRock Polychrome USB</li>
      <li><i></i>ASUS Aura Sync <span>скоро</span></li>
      <li><i></i>OpenRGB <span>скоро</span></li>
    </ul>
  </footer>
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border-right: 1px solid var(--border);
    width: 270px;
    padding: 22px 18px;
    min-height: 0;
    overflow: hidden;
    white-space: nowrap;
    transition: width 0.2s, padding 0.2s;
  }
  aside.collapsed { width: 72px; padding: 22px 12px; }
  .collapsed .text { display: none; }
  .collapsed .status { justify-content: center; }
  header { display: flex; align-items: center; gap: 12px; padding: 0 4px 34px; }
  header b { font-size: 20px; font-weight: 600; }
  header small { margin-left: auto; color: var(--dim); align-self: flex-start; }
  nav { display: flex; flex-direction: column; gap: 6px; }
  nav button, .fold {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 13px 14px;
    border: 0;
    border-radius: 10px;
    background: none;
    font-size: 16px;
    color: #c4ccda;
    text-align: left;
  }
  nav button:hover, .fold:hover { background: var(--card); }
  .fold { width: 100%; margin-bottom: 12px; font-size: 14px; color: var(--dim); }
  nav button.active {
    background: linear-gradient(90deg, rgba(47, 124, 246, 0.28), rgba(47, 124, 246, 0.1));
    color: #5aa0ff;
    box-shadow: inset 3px 0 0 var(--accent);
  }
  footer { margin-top: auto; }
  .status { display: flex; align-items: center; gap: 10px; margin: 0 0 18px; padding-bottom: 18px; border-bottom: 1px solid var(--border); }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 14px; color: #c4ccda; font-size: 13px; }
  li { display: flex; align-items: center; gap: 10px; }
  li span { margin-left: auto; color: var(--faint); font-size: 12px; }
  i { width: 9px; height: 9px; border-radius: 50%; background: #3a4456; flex: none; }
  i.ok { background: var(--ok); box-shadow: 0 0 8px var(--ok); }
  i.error { background: var(--bad); }
  i.connecting { background: #e5b33a; }
</style>
