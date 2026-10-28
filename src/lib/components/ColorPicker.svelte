<script lang="ts">
  import { PRESETS, css, hex, hsv, toHsv, fromHex, type RGB } from '$lib/leds';

  /** Круг, HEX, недавние и готовые цвета. sethex — для HEX, если его надо разбирать иначе (по умолчанию set) */
  let { color, set, sethex = set }: { color: RGB; set: (c: RGB) => void; sethex?: (c: RGB) => void } = $props();

  const SIZE = 130;
  const RECENT_KEY = 'polychrome.recent';
  let wheel = $state<HTMLCanvasElement>();
  let hexText = $state('');
  let recent = $state<RGB[]>(loadRecent());

  const marker = $derived.by(() => {
    const [h, s] = toHsv(color);
    const a = h * Math.PI * 2 - Math.PI / 2;
    return { x: SIZE / 2 + Math.cos(a) * s * (SIZE / 2), y: SIZE / 2 + Math.sin(a) * s * (SIZE / 2) };
  });

  $effect(() => {
    hexText = hex(color);
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

  function pick(e: PointerEvent) {
    if (!wheel || (e.type === 'pointermove' && !e.buttons)) return;
    const rect = wheel.getBoundingClientRect();
    const dx = e.clientX - rect.left - rect.width / 2;
    const dy = e.clientY - rect.top - rect.height / 2;
    const s = Math.min(Math.hypot(dx, dy) / (rect.width / 2), 1);
    const h = ((Math.atan2(dy, dx) + Math.PI / 2) / (Math.PI * 2) + 1) % 1;
    set(hsv(h, s, 1));
  }

  function commitHex() {
    const m = hexText.trim().match(/^#?([0-9a-f]{6})$/i);
    if (!m) return void (hexText = hex(color));
    sethex(fromHex('#' + m[1]));
    remember(color);
  }
</script>

{#snippet swatches(list: RGB[], label: string)}
  <div class="swatches" role="group" aria-label={label}>
    {#each list as c (c.join())}
      <button class:on={c.join() === color.join()} style:background={css(c)} aria-label="Цвет {hex(c)}" title={hex(c)} onclick={() => set(c)}></button>
    {/each}
  </div>
{/snippet}

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
      onpointerup={() => remember(color)}
    ></canvas>
    <span style:left="{marker.x}px" style:top="{marker.y}px" style:background={css(color)}></span>
  </div>
  <div class="side">
    <label class="hex">
      <i style:background={css(color)}></i>
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

<style>
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
</style>
