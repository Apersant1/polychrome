<script lang="ts">
  import Check from '@lucide/svelte/icons/check';
  import { app, change } from './app.svelte';
  import EffectStrip from './EffectStrip.svelte';
  import { MODES, HINTS, type RGB } from './leds';

  /** all — показать все группы сразу (страница «Эффекты») */
  let { all = false }: { all?: boolean } = $props();

  const MUSIC = -1; // режим 6 ведёт хост по захвату звука — пока не сделан
  const TABS: [string, number[]][] = [
    ['Популярные', [1, 2, 14, 7, 3, MUSIC]],
    ['Дополнительные', [4, 5, 8, 9, 10, 11, 12, 13, 15, 0]],
    ['Пользовательские', [16, 17]],
  ];
  const WAVY = [7, 13, 8, 15];
  let tab = $state(0);

  const preview = (mode: number) => ({ ...app.edit, mode, palette: app.edit.palette.length ? app.edit.palette : DEFAULT_PALETTE });
  const DEFAULT_PALETTE: RGB[] = [[255, 0, 80], [140, 40, 255], [0, 140, 255]];

  function choose(mode: number) {
    if (mode === MUSIC) return;
    change({ mode, palette: app.edit.palette.length >= 2 ? app.edit.palette : DEFAULT_PALETTE });
  }
</script>

{#snippet cards(modes: number[])}
  <div class="grid">
    {#each modes as mode (mode)}
      <button class="card" class:on={app.edit.mode === mode} disabled={mode === MUSIC} onclick={() => choose(mode)}>
        {#if app.edit.mode === mode}<span class="tick"><Check size={12} strokeWidth={3} /></span>{/if}
        {#if mode === MUSIC}
          <div class="bars">{#each [3, 6, 4, 8, 5, 9, 4, 7, 3, 6, 8, 4] as b, i (i)}<i style:height="{b * 4}px"></i>{/each}</div>
          <b>Музыка</b><small>Реакция на звук · скоро</small>
        {:else}
          <EffectStrip look={preview(mode)} wave={WAVY.includes(mode) ? 1 : 0} />
          <b>{MODES[mode]}</b><small>{HINTS[mode]}</small>
        {/if}
      </button>
    {/each}
  </div>
{/snippet}

<div class="panel effects">
  {#if all}
    {#each TABS as [title, modes] (title)}
      <h3>{title}</h3>
      {@render cards(modes)}
    {/each}
  {:else}
    <div class="tabs" role="tablist">
      {#each TABS as [title], i (title)}
        <button role="tab" aria-selected={tab === i} class:on={tab === i} onclick={() => (tab = i)}>{title}</button>
      {/each}
    </div>
    {@render cards(TABS[tab][1])}
  {/if}
</div>

<style>
  .effects { overflow: auto; padding-top: 6px; }
  .effects > h3 { margin-top: 16px; }
  .tabs { display: flex; gap: 24px; border-bottom: 1px solid var(--border); margin-bottom: 14px; white-space: nowrap; }
  .tabs button { padding: 12px 4px; border: 0; background: none; color: var(--dim); box-shadow: inset 0 -2px 0 transparent; }
  .tabs button.on { color: var(--text); font-weight: 600; box-shadow: inset 0 -2px 0 var(--accent); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 10px; }
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 10px 10px 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--card);
    text-align: left;
  }
  .card:hover:not(:disabled) { background: var(--card-hover); }
  .card.on { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent), 0 0 18px rgba(47, 124, 246, 0.25); }
  .card:disabled { opacity: 0.55; }
  .card b { margin-top: 9px; font-weight: 600; }
  .card small { color: var(--dim); font-size: 12px; }
  .tick {
    position: absolute;
    top: 6px;
    left: 6px;
    z-index: 1;
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--accent);
    color: #fff;
  }
  .bars { display: flex; align-items: center; justify-content: center; gap: 5px; height: 44px; border-radius: 6px; background: #080c13; }
  .bars i { width: 5px; border-radius: 2px; background: linear-gradient(#ff4fd8, #6b5bff); }
</style>
