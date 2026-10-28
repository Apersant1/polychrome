<script lang="ts">
  import Sun from '@lucide/svelte/icons/sun';
  import Gauge from '@lucide/svelte/icons/gauge';
  import Plus from '@lucide/svelte/icons/plus';
  import X from '@lucide/svelte/icons/x';
  import CopyCheck from '@lucide/svelte/icons/copy-check';
  import ColorPicker from './ColorPicker.svelte';
  import { app } from '$lib/state/app.svelte';
  import { change, setPower, toAll, zoneOf, zoneLabel } from '$lib/state/board.svelte';
  import { MODES, MAX_PALETTE, MODE_OFF, css, hex, split, usesColor, usesPalette, animated, type RGB } from '$lib/leds';

  let sel = $state(0); // какой цвет палитры правим

  const zone = $derived(zoneOf(app.selected));
  const mode = $derived(app.edit.mode);
  const palette = $derived(usesPalette(mode));
  const single = $derived(usesColor(mode) && !palette);
  const current = $derived<RGB>(palette ? (app.edit.palette[sel] ?? app.edit.color) : app.edit.color);

  $effect(() => {
    if (sel >= app.edit.palette.length) sel = 0;
  });

  /** Цвет полной яркости: в палитру — в выбранный образец, иначе — цвет зоны */
  function setColor(c: RGB) {
    if (palette) change({ palette: app.edit.palette.map((p, i) => (i === sel ? c : p)) });
    else change({ color: c });
  }

  // Тёмный цвет из HEX раскладываем на оттенок и яркость, чтобы на железе вышло ровно то, что ввели
  const setHex = (c: RGB) => (palette ? setColor(c) : change(split(c)));

  function addColor() {
    change({ palette: [...app.edit.palette, app.edit.palette.at(-1) ?? [255, 255, 255]] });
    sel = app.edit.palette.length - 1;
  }

  const removeColor = (i: number) => change({ palette: app.edit.palette.filter((_, k) => k !== i) });
</script>

{#snippet slider(id: string, label: string, Icon: typeof Sun, value: number, key: 'bright' | 'speed')}
  <div class="row">
    <label for={id}><Icon size={16} />{label}</label>
    <input {id} type="range" min="0" max="100" {value} style:--fill="{value}%" oninput={(e) => change({ [key]: +e.currentTarget.value })} />
    <b>{value}%</b>
  </div>
{/snippet}

<div class="panel">
  <h3>
    Цвет{#if zone}<span>· {zoneLabel(zone)}</span>{/if}
    {#if zone && mode !== MODE_OFF}
      <button class="all" disabled={app.status !== 'ok'} title="Такой же цвет и режим на все зоны" onclick={toAll}><CopyCheck size={14} />На все</button>
    {/if}
  </h3>

  {#if !zone}
    <p class="empty">Выберите зону слева</p>
  {:else if mode === MODE_OFF}
    <div class="empty">
      <p>Зона выключена</p>
      <button class="btn primary" onclick={() => setPower([zone.id], true)}>Включить</button>
    </div>
  {:else}
    <p class="mode">Режим: <b>{MODES[mode] ?? `#${mode}`}</b></p>

    {#if palette}
      <div class="palette" role="group" aria-label="Палитра эффекта">
        {#each app.edit.palette as c, i (i)}
          <div class="chip" class:on={i === sel}>
            <button style:background={css(c)} aria-label="Цвет палитры {i + 1}: {hex(c)}" aria-pressed={i === sel} onclick={() => (sel = i)}></button>
            {#if app.edit.palette.length > 2}
              <button class="drop" aria-label="Убрать цвет {i + 1}" onclick={() => removeColor(i)}><X size={10} /></button>
            {/if}
          </div>
        {/each}
        {#if app.edit.palette.length < MAX_PALETTE}
          <button class="add" aria-label="Добавить цвет в палитру" onclick={addColor}><Plus size={14} /></button>
        {/if}
      </div>
    {/if}

    {#if single || palette}
      <ColorPicker color={current} set={setColor} sethex={setHex} />
      {@render slider('bright', 'Яркость', Sun, app.edit.bright, 'bright')}
    {:else}
      <p class="note">Цвета задаёт сам эффект — здесь настраивается только скорость.</p>
    {/if}

    {#if animated(mode)}{@render slider('speed', 'Скорость', Gauge, app.edit.speed, 'speed')}{/if}
  {/if}
</div>

<style>
  .panel { display: flex; flex-direction: column; gap: 14px; overflow: auto; }
  h3 { display: flex; align-items: center; margin-bottom: 0; }
  h3 span { margin-left: 6px; min-width: 0; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; color: var(--dim); font-weight: 400; }
  .all { display: flex; align-items: center; gap: 6px; flex: none; margin-left: auto; padding: 4px 8px; border: 1px solid var(--border-strong); border-radius: 6px; background: none; color: var(--dim); font: inherit; font-size: 12px; font-weight: 500; }
  .all:hover:not(:disabled) { color: var(--text); background: var(--card); }
  .mode { margin: 0; color: var(--dim); font-size: 13px; }
  .mode b { color: var(--text); font-weight: 500; }
  .empty { display: grid; justify-items: center; gap: 12px; margin: auto 0; color: var(--dim); text-align: center; }
  .empty p { margin: 0; }
  .note { margin: 0; color: var(--dim); font-size: 13px; line-height: 1.5; }
  .palette { display: flex; flex-wrap: wrap; gap: 10px; }
  .chip { position: relative; }
  .chip > button:first-child { width: 30px; height: 30px; padding: 0; border-radius: 8px; border: 1px solid rgba(255, 255, 255, 0.2); }
  .chip.on > button:first-child { box-shadow: 0 0 0 2px var(--panel), 0 0 0 4px var(--accent); }
  .drop {
    position: absolute;
    top: -6px;
    right: -6px;
    display: none;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: #39445a;
  }
  .chip:hover .drop, .chip.on .drop, .drop:focus-visible { display: grid; }
  .add { display: grid; place-items: center; width: 30px; height: 30px; border-radius: 8px; border: 1px dashed var(--border-strong); background: none; color: var(--dim); }
  .row { display: flex; align-items: center; gap: 12px; }
  .row label { display: flex; align-items: center; gap: 8px; flex: none; width: 92px; color: var(--dim); font-size: 13px; }
  .row b { width: 40px; text-align: right; color: var(--text); font-weight: 500; }
</style>
