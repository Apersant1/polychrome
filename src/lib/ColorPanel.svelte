<script lang="ts">
  import Sun from '@lucide/svelte/icons/sun';
  import Gauge from '@lucide/svelte/icons/gauge';
  import Plus from '@lucide/svelte/icons/plus';
  import X from '@lucide/svelte/icons/x';
  import { app, change, setPower, zoneOf, zoneLabel } from './app.svelte';
  import { MODES, PRESETS, MAX_PALETTE, MODE_OFF, css, hex, hsv, toHsv, split, usesColor, usesPalette, animated, type RGB } from './leds';

  const SIZE = 130;
  const RECENT_KEY = 'polychrome.recent';
  let wheel = $state<HTMLCanvasElement>();
  let sel = $state(0); // какой цвет палитры правим
  let hexText = $state('');
  let recent = $state<RGB[]>(loadRecent());

  const zone = $derived(zoneOf(app.selected));
  const mode = $derived(app.edit.mode);
  const palette = $derived(usesPalette(mode));
  const single = $derived(usesColor(mode) && !palette);
  const current = $derived<RGB>(palette ? (app.edit.palette[sel] ?? app.edit.color) : app.edit.color);
  const marker = $derived.by(() => {
    const [h, s] = toHsv(current);
    const a = h * Math.PI * 2 - Math.PI / 2;
    return { x: SIZE / 2 + Math.cos(a) * s * (SIZE / 2), y: SIZE / 2 + Math.sin(a) * s * (SIZE / 2) };
  });

  $effect(() => {
    if (sel >= app.edit.palette.length) sel = 0;
  });
  $effect(() => {
    hexText = hex(current);
  });

  $effect(() => {
    if (!wheel) return;
    const c = wheel.getContext('2d')!;
    const r = SIZE / 2;
    const hue = c.createConicGradient(-Math.PI / 2, r, r);
    for (let i = 0; i <= 12; i++) hue.addColorStop(i / 12, css(hsv((i / 12) % 1, 1, 1)));
    c.fillStyle = hue;
    c.beginPath();
    c.arc(r, r, r, 0, Math.PI * 2);
    c.fill();
    const white = c.createRadialGradient(r, r, 0, r, r, r);
    white.addColorStop(0, '#fff');
    white.addColorStop(1, 'rgba(255,255,255,0)');
    c.fillStyle = white;
    c.fill();
  });

  function loadRecent(): RGB[] {
    try {
      return JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]');
    } catch {
      return [];
    }
  }

  function remember(c: RGB) {
    recent = [c, ...recent.filter((r) => r.join() !== c.join())].slice(0, 8);
    try {
      localStorage.setItem(RECENT_KEY, JSON.stringify(recent));
    } catch {
      // не критично: просто не запомним
    }
  }

  /** Цвет полной яркости: в палитру — в выбранный образец, иначе — цвет зоны */
  function setColor(c: RGB) {
    if (palette) change({ palette: app.edit.palette.map((p, i) => (i === sel ? c : p)) });
    else change({ color: c });
  }

  function pick(e: PointerEvent) {
    if (!wheel || (e.type === 'pointermove' && !e.buttons)) return;
    const rect = wheel.getBoundingClientRect();
    const dx = e.clientX - rect.left - rect.width / 2;
    const dy = e.clientY - rect.top - rect.height / 2;
    const s = Math.min(Math.hypot(dx, dy) / (rect.width / 2), 1);
    const h = ((Math.atan2(dy, dx) + Math.PI / 2) / (Math.PI * 2) + 1) % 1;
    setColor(hsv(h, s, 1));
  }

  function commitHex() {
    const m = hexText.trim().match(/^#?([0-9a-f]{6})$/i);
    if (!m) return void (hexText = hex(current));
    const rgb = [0, 2, 4].map((i) => parseInt(m[1].slice(i, i + 2), 16)) as RGB;
    // Тёмный цвет из HEX раскладываем на оттенок и яркость, чтобы на железе вышло ровно то, что ввели
    if (palette) setColor(rgb);
    else change(split(rgb));
    remember(palette ? rgb : app.edit.color);
  }

  function addColor() {
    change({ palette: [...app.edit.palette, app.edit.palette.at(-1) ?? [255, 255, 255]] });
    sel = app.edit.palette.length - 1;
  }

  const removeColor = (i: number) => change({ palette: app.edit.palette.filter((_, k) => k !== i) });
</script>

{#snippet swatches(list: RGB[], label: string)}
  <div class="swatches" role="group" aria-label={label}>
    {#each list as c (c.join())}
      <button class:on={c.join() === current.join()} style:background={css(c)} aria-label="Цвет {hex(c)}" title={hex(c)} onclick={() => setColor(c)}></button>
    {/each}
  </div>
{/snippet}

{#snippet slider(id: string, label: string, Icon: typeof Sun, value: number, key: 'bright' | 'speed')}
  <div class="row">
    <label for={id}><Icon size={16} />{label}</label>
    <input {id} type="range" min="0" max="100" {value} style:--fill="{value}%" oninput={(e) => change({ [key]: +e.currentTarget.value })} />
    <b>{value}%</b>
  </div>
{/snippet}

<div class="panel">
  <h3>Цвет{#if zone}<span>· {zoneLabel(zone)}</span>{/if}</h3>

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
      <div class="picker">
        <div class="wheel">
          <canvas
            bind:this={wheel}
            width={SIZE}
            height={SIZE}
            onpointerdown={(e) => {
              wheel!.setPointerCapture(e.pointerId);
              pick(e);
            }}
            onpointermove={pick}
            onpointerup={() => remember(current)}
          ></canvas>
          <span style:left="{marker.x}px" style:top="{marker.y}px" style:background={css(current)}></span>
        </div>
        <div class="side">
          <label class="hex">
            <i style:background={css(current)}></i>
            <input
              aria-label="Цвет в HEX"
              spellcheck="false"
              maxlength="7"
              bind:value={hexText}
              onkeydown={(e) => e.key === 'Enter' && commitHex()}
              onblur={commitHex}
            />
          </label>
          {#if recent.length}<small>Недавние</small>{@render swatches(recent, 'Недавние цвета')}{/if}
        </div>
      </div>
      {@render swatches(PRESETS, 'Готовые цвета')}
      {@render slider('bright', 'Яркость', Sun, app.edit.bright, 'bright')}
    {:else}
      <p class="note">Цвета задаёт сам эффект — здесь настраивается только скорость.</p>
    {/if}

    {#if animated(mode)}{@render slider('speed', 'Скорость', Gauge, app.edit.speed, 'speed')}{/if}
  {/if}
</div>

<style>
  .panel { display: flex; flex-direction: column; gap: 14px; overflow: auto; }
  h3 { margin-bottom: 0; }
  h3 span { margin-left: 6px; color: var(--dim); font-weight: 400; }
  .mode { margin: 0; color: var(--dim); font-size: 13px; }
  .mode b { color: var(--text); font-weight: 500; }
  .empty { display: grid; justify-items: center; gap: 12px; margin: auto 0; color: var(--dim); text-align: center; }
  .empty p { margin: 0; }
  .note { margin: 0; color: var(--dim); font-size: 13px; line-height: 1.5; }
  .picker { display: flex; gap: 16px; align-items: flex-start; }
  .wheel { position: relative; flex: none; width: 130px; height: 130px; }
  canvas { display: block; border-radius: 50%; cursor: crosshair; touch-action: none; }
  .wheel span {
    position: absolute;
    width: 18px;
    height: 18px;
    margin: -9px 0 0 -9px;
    border-radius: 50%;
    border: 3px solid #fff;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.4), 0 2px 6px rgba(0, 0, 0, 0.5);
    pointer-events: none;
  }
  .side { display: grid; gap: 8px; min-width: 0; flex: 1; }
  .side small { color: var(--dim); font-size: 12px; }
  .hex { display: flex; align-items: center; gap: 8px; padding: 6px 8px; border: 1px solid var(--border-strong); border-radius: 8px; background: var(--card); }
  .hex i { width: 18px; height: 18px; border-radius: 4px; flex: none; }
  .hex input { width: 100%; min-width: 0; border: 0; background: none; color: var(--text); font: inherit; font-family: ui-monospace, monospace; text-transform: uppercase; outline: none; }
  .hex:focus-within { border-color: var(--accent); }
  .swatches { display: flex; flex-wrap: wrap; gap: 8px; }
  .swatches button { width: 22px; height: 22px; border-radius: 50%; border: 0; padding: 0; }
  .swatches button.on { box-shadow: 0 0 0 2px var(--panel), 0 0 0 4px #fff; }
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
