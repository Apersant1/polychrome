<script lang="ts">
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import Check from '@lucide/svelte/icons/check';
  import Plus from '@lucide/svelte/icons/plus';
  import Save from '@lucide/svelte/icons/save';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import Settings2 from '@lucide/svelte/icons/settings-2';
  import { app } from '$lib/state/app.svelte';
  import { useProfile, saveProfile, profileChanged, profileExists, profileSwatch } from '$lib/state/profiles.svelte';

  let menu: HTMLDivElement;
  let naming = $state(false);
  let name = $state('');

  const ready = $derived(app.status === 'ok');
  const current = $derived(app.profiles.find((p) => p.name === app.profile));
  const changed = $derived(!!current && profileChanged(current));
  const trimmed = $derived(name.trim());
  const taken = $derived(profileExists(trimmed));

  /** Пункт меню: выполнить и закрыть */
  function pick(run: () => void) {
    menu.hidePopover();
    run();
  }

  function create() {
    if (trimmed && !taken) pick(() => saveProfile(trimmed));
  }

  // Меню закрылось — недописанное имя не храним
  function toggled(e: ToggleEvent) {
    if (e.newState === 'closed') [naming, name] = [false, ''];
  }
</script>

<button class="trigger" popovertarget="profile-menu" title={changed ? 'Настройки изменены после выбора профиля' : 'Профиль'}>
  {#if current}<span class="swatch" style:background={profileSwatch(current)}></span>{/if}
  <span class="label">{current?.name ?? 'Профиль…'}</span>
  {#if changed}<i class="dot"></i>{/if}
  <ChevronDown size={16} />
</button>

<div class="menu" id="profile-menu" popover bind:this={menu} ontoggle={toggled}>
  {#each app.profiles as p (p.name)}
    <button disabled={!ready} title="Включить и записать в контроллер" onclick={() => pick(() => useProfile(p.name))}>
      <span class="swatch" style:background={profileSwatch(p)}></span>
      <span class="label">{p.name}</span>
      {#if p.name === app.profile}<Check size={14} />{/if}
    </button>
  {/each}

  {#if changed && current}
    <hr />
    <button onclick={() => pick(() => saveProfile(current.name))}><Save size={14} />Сохранить в «{current.name}»</button>
    <button disabled={!ready} onclick={() => pick(() => useProfile(current.name))}><RotateCcw size={14} />Вернуть как в профиле</button>
  {/if}

  <hr />
  {#if naming}
    <!-- svelte-ignore a11y_autofocus -->
    <input
      class="name"
      class:taken
      placeholder="Название профиля"
      aria-invalid={taken}
      bind:value={name}
      autofocus
      onkeydown={(e) => (e.key === 'Enter' ? create() : e.key === 'Escape' && (e.stopPropagation(), (naming = false)))}
    />
    {#if taken}<small>Такое имя уже есть</small>{/if}
  {:else}
    <button onclick={() => (naming = true)}><Plus size={14} />Новый профиль из текущего</button>
  {/if}
  <button onclick={() => pick(() => (app.page = 'profiles'))}><Settings2 size={14} />Все профили…</button>
</div>

<style>
  .trigger {
    anchor-name: --profile-menu;
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 200px;
    padding: 8px 12px;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    background: var(--card);
    color: var(--text);
    font: inherit;
  }
  .trigger:hover { background: var(--card-hover); }
  .trigger .label { flex: 1; text-align: left; }
  .trigger :global(svg) { color: var(--dim); }
  .label { overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  .swatch { flex: none; width: 28px; height: 14px; border-radius: 4px; filter: saturate(1.2); }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: var(--warn); flex: none; }
  .menu {
    position-anchor: --profile-menu;
    position-area: bottom span-left;
    margin: 6px 0 0;
    padding: 4px;
    width: 260px;
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    background: var(--card-hover);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    color: var(--text);
  }
  .menu button {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: inherit;
    font: inherit;
    font-size: 13px;
    text-align: left;
  }
  .menu button .label { flex: 1; }
  .menu button:hover:not(:disabled), .menu button:focus-visible { background: rgba(255, 255, 255, 0.08); }
  .menu button:disabled { opacity: 0.5; }
  .menu :global(svg) { flex: none; color: var(--dim); }
  hr { margin: 4px 6px; border: 0; border-top: 1px solid var(--border); }
  .name {
    width: 100%;
    padding: 7px 10px;
    border: 1px solid var(--accent);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font: inherit;
    font-size: 13px;
    outline: none;
  }
  .name.taken { border-color: var(--warn); }
  small { display: block; padding: 4px 10px 2px; color: var(--warn); font-size: 11px; }
</style>
