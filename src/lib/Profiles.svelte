<script lang="ts">
  import Plus from '@lucide/svelte/icons/plus';
  import Ellipsis from '@lucide/svelte/icons/ellipsis';
  import Save from '@lucide/svelte/icons/save';
  import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
  import Pencil from '@lucide/svelte/icons/pencil';
  import Trash from '@lucide/svelte/icons/trash';
  import {
    app,
    useProfile,
    saveProfile,
    renameProfile,
    deleteProfile,
    profileChanged,
    profileExists,
    profileLook,
    type Profile,
  } from './app.svelte';
  import { actual, css, hsv, usesColor, usesPalette, MODE_OFF } from './leds';

  /** grid — крупные карточки во всю страницу («Профили») */
  let { grid = false }: { grid?: boolean } = $props();
  let naming = $state(false);
  let renaming = $state(''); // имя профиля, который сейчас переименовываем
  let name = $state('');
  let undo = $state<{ name: string; restore: () => void }>();
  let undoTimer: ReturnType<typeof setTimeout>;

  const ready = $derived(app.status === 'ok');
  const trimmed = $derived(name.trim());
  const taken = $derived(trimmed !== renaming && profileExists(trimmed));

  /** Полоска цветов профиля по зонам: статичная подсказка, а не анимация */
  function swatch(p: Profile) {
    const stops = app.zones.flatMap((z) => {
      const l = profileLook(p, z.id);
      if (!l || l.mode === MODE_OFF) return ['#1a212c'];
      if (usesPalette(l.mode)) return actual(l).palette.map((c) => css(c));
      if (!usesColor(l.mode)) return [0, 0.33, 0.66].map((h) => css(hsv(h, 1, 1)));
      return [css(actual(l).rgb)];
    });
    return `linear-gradient(90deg, ${(stops.length > 1 ? stops : [...stops, ...stops]).join(', ')})`;
  }

  function startRename(p: Profile) {
    renaming = p.name;
    name = p.name;
  }

  /** Enter/blur: создать или переименовать; занятое имя не принимаем */
  function commit() {
    if (trimmed && !taken) {
      if (renaming) renameProfile(renaming, trimmed);
      else saveProfile(trimmed);
    }
    cancel();
  }

  function cancel() {
    naming = false;
    renaming = '';
    name = '';
  }

  function remove(p: Profile) {
    const restore = deleteProfile(p.name);
    if (!restore) return;
    clearTimeout(undoTimer);
    undo = { name: p.name, restore };
    undoTimer = setTimeout(() => (undo = undefined), 6000);
  }

  function restore() {
    undo?.restore();
    undo = undefined;
    clearTimeout(undoTimer);
  }

  /** Пункт меню: выполнить и закрыть меню */
  const pick = (e: MouseEvent, run: () => void) => {
    (e.currentTarget as HTMLElement).closest<HTMLElement>('[popover]')?.hidePopover();
    run();
  };
</script>

{#snippet nameInput(placeholder: string)}
  <!-- svelte-ignore a11y_autofocus -->
  <input
    class="name"
    class:taken
    {placeholder}
    bind:value={name}
    autofocus
    aria-invalid={taken}
    title={taken ? 'Такое имя уже есть' : ''}
    onkeydown={(e) => (e.key === 'Enter' ? commit() : e.key === 'Escape' && cancel())}
    onblur={commit}
  />
{/snippet}

<div class="panel profiles" class:grid>
  <h3>Профили</h3>
  <div class="list">
    {#if naming}
      <div class="new">{@render nameInput('Название профиля')}{#if taken}<small>Такое имя уже есть</small>{/if}</div>
    {:else}
      <button class="new" onclick={() => (naming = true)}><Plus size={18} />Новый профиль</button>
    {/if}
    {#each app.profiles as p, i (p.name)}
      {@const active = app.profile === p.name}
      {@const changed = active && profileChanged(p)}
      <div class="card" class:on={active} style:anchor-name="--profile-{i}">
        {#if renaming === p.name}
          <span class="swatch" style:background={swatch(p)}></span>
          {@render nameInput('Новое имя')}
        {:else}
          <button class="use" onclick={() => useProfile(p.name)} disabled={!ready} title="Включить и записать в контроллер">
            <span class="swatch" style:background={swatch(p)}></span>
            <span class="title">{p.name}{#if changed}<i title="Настройки изменены после выбора профиля"></i>{/if}</span>
          </button>
          <div class="acts">
            {#if changed}
              <button aria-label="Обновить профиль текущими настройками" title="Обновить профиль" onclick={() => saveProfile(p.name)}><Save size={14} /></button>
              <button aria-label="Вернуть настройки профиля" title="Вернуть как в профиле" disabled={!ready} onclick={() => useProfile(p.name)}><RotateCcw size={14} /></button>
            {/if}
            <button aria-label="Действия с профилем {p.name}" title="Ещё" popovertarget="profile-menu-{i}"><Ellipsis size={16} /></button>
          </div>
          <div class="menu" id="profile-menu-{i}" popover style:position-anchor="--profile-{i}">
            <button onclick={(e) => pick(e, () => startRename(p))}><Pencil size={14} />Переименовать</button>
            <button onclick={(e) => pick(e, () => saveProfile(p.name))}><Save size={14} />Обновить из текущего</button>
            <button class="danger" onclick={(e) => pick(e, () => remove(p))}><Trash size={14} />Удалить</button>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

{#if undo}
  <div class="toast" role="status">
    Профиль «{undo.name}» удалён
    <button onclick={restore}>Вернуть</button>
  </div>
{/if}

<style>
  .profiles { display: flex; align-items: center; gap: 24px; padding: 14px 20px; }
  .profiles > h3 { margin: 0; }
  .list { display: flex; gap: 14px; overflow-x: auto; flex: 1; padding: 2px; }
  .grid { flex-direction: column; align-items: stretch; }
  .grid .list { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); overflow: visible; }
  .new {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 4px;
    flex: none;
    width: 180px;
    min-height: 64px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--card);
    color: var(--text);
    font: inherit;
  }
  button.new { flex-direction: row; justify-content: flex-start; align-items: center; gap: 12px; padding: 0 16px; }
  div.new { padding: 6px 8px; }
  div.new small { color: var(--warn); font-size: 11px; padding-left: 2px; }
  .name {
    width: 100%;
    padding: 6px 8px;
    border: 1px solid var(--accent);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
    font: inherit;
    outline: none;
  }
  .name.taken { border-color: var(--warn); }
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: none;
    width: 180px;
    padding: 8px 10px 9px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--card);
  }
  .card:hover { background: var(--card-hover); }
  .card.on { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }
  .grid .card { width: auto; }
  .use { display: flex; flex-direction: column; gap: 8px; padding: 0; border: 0; background: none; text-align: left; font: inherit; color: inherit; }
  .card.on .title { color: #5aa0ff; }
  .title { display: flex; align-items: center; gap: 6px; padding-right: 76px; min-height: 22px; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }
  .title i { width: 7px; height: 7px; border-radius: 50%; background: var(--warn); flex: none; }
  .swatch { display: block; height: 26px; border-radius: 6px; filter: saturate(1.2); }
  .grid .swatch { height: 60px; }
  .acts { position: absolute; right: 6px; bottom: 6px; display: flex; gap: 2px; }
  .acts button {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--dim);
  }
  .acts button:hover:not(:disabled) { background: rgba(255, 255, 255, 0.08); color: var(--text); }
  .menu {
    position-area: bottom span-left;
    margin: 4px 0 0;
    padding: 4px;
    min-width: 200px;
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
    font: inherit;
    font-size: 13px;
    text-align: left;
  }
  .menu button:hover, .menu button:focus-visible { background: rgba(255, 255, 255, 0.08); }
  .menu .danger { color: var(--bad); }
  .toast {
    position: fixed;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 12px 10px 16px;
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    background: var(--card-hover);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    font-size: 13px;
    z-index: 10;
  }
  .toast button { padding: 6px 10px; border: 0; border-radius: 6px; background: var(--accent-soft); color: #5aa0ff; font: inherit; font-weight: 600; }
</style>
