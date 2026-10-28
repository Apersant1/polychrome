<script lang="ts">
  import House from '@lucide/svelte/icons/house';
  import CircuitBoard from '@lucide/svelte/icons/circuit-board';
  import Sparkles from '@lucide/svelte/icons/sparkles';
  import User from '@lucide/svelte/icons/user';
  import Settings from '@lucide/svelte/icons/settings';
  import { app, type Page } from '$lib/state/app.svelte';

  const NAV: [Page, string, typeof House][] = [
    ['home', 'Главная', House],
    ['devices', 'Устройства', CircuitBoard],
    ['effects', 'Эффекты', Sparkles],
    ['profiles', 'Профили', User],
    ['settings', 'Настройки', Settings],
  ];

  const status = $derived(
    app.status === 'ok' ? 'Контроллер подключён' : app.status === 'connecting' ? 'Подключение…' : 'Нет связи с контроллером',
  );
</script>

<aside>
  <header data-tauri-drag-region>
    <svg width="34" height="30" viewBox="0 0 34 30" aria-hidden="true">
      <defs>
        <linearGradient id="logo" x1="0" y1="1" x2="1" y2="0">
          <stop offset="0" stop-color="#ff3d6e" /><stop offset="0.5" stop-color="#a64dff" /><stop offset="1" stop-color="#2f9bff" />
        </linearGradient>
      </defs>
      <path d="M17 2 32 28H2Z" fill="none" stroke="url(#logo)" stroke-width="4" stroke-linejoin="round" />
    </svg>
    <b>Polychrome</b>
    <small>v0.1.0</small>
  </header>

  <nav>
    {#each NAV as [page, label, Icon] (page)}
      <button class:active={app.page === page} onclick={() => (app.page = page)}>
        <Icon size={20} />{label}
      </button>
    {/each}
  </nav>

  <footer>
    <p class="status"><i class={app.status}></i>{status}</p>
    <ul>
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
    padding: 22px 18px;
    min-height: 0;
  }
  header { display: flex; align-items: center; gap: 12px; padding: 0 4px 34px; }
  header b { font-size: 20px; font-weight: 600; }
  header small { margin-left: auto; color: var(--dim); align-self: flex-start; }
  nav { display: flex; flex-direction: column; gap: 6px; }
  nav button {
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
  nav button:hover { background: var(--card); }
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
