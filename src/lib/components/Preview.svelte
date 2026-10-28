<script lang="ts">
  import Pencil from '@lucide/svelte/icons/pencil';
  import Check from '@lucide/svelte/icons/check';
  import { app, type PartZone } from '$lib/state/app.svelte';
  import { FANS_ZONE, select, zoneLabel } from '$lib/state/board.svelte';
  import { setFanCount, setPartZone } from '$lib/state/config.svelte';
  import { placeholders } from '$lib/state/system.svelte';
  import { led, css, mix, type Look, type RGB } from '$lib/leds';

  // Сцена 800×560, растягивается по месту с сохранением пропорций
  const W = 800, H = 560;
  type Box = [x: number, y: number, w: number, h: number];
  type Part = { label: string; box?: Box; zone: PartZone; strips?: [number, number, number, number][] }; // у групп вентиляторов рамку строит boxOf
  // Детали картинки: где лежат (для наведения и клика) и какой зоной светятся по умолчанию — как на B760M Pro RS/D4.
  // Пользователь перепривязывает их в режиме «Настроить» (app.scene); у лент — отрезки [x0, y0, x1, y1].
  const PARTS: Record<string, Part> = {
    left: { label: 'Лента слева', box: [34, 44, 20, 432], zone: 0, strips: [[44, 50, 44, 470]] },
    top: { label: 'Лента сверху', box: [64, 18, 682, 18], zone: 1, strips: [[70, 28, 740, 28]] },
    bottom: { label: 'Лента снизу', box: [44, 481, 712, 18], zone: 3, strips: [[50, 490, 750, 490]] },
    right: { label: 'Лента справа', box: [749, 64, 18, 412], zone: 4, strips: [[758, 70, 758, 470]] },
    io: { label: 'Кожух I/O', box: [174, 104, 70, 100], zone: 5, strips: [[182, 112, 182, 196], [182, 112, 236, 112]] },
    corner: { label: 'Уголок платы', box: [482, 98, 100, 80], zone: 6, strips: [[490, 106, 574, 106], [574, 106, 574, 170]] },
    topFans: { label: 'Верхние вентиляторы', zone: FANS_ZONE },
    frontFans: { label: 'Передние вентиляторы', zone: FANS_ZONE },
    rearFan: { label: 'Задний вентилятор', box: [57, 102, 96, 96], zone: FANS_ZONE },
    cpu: { label: 'Кулер процессора', box: [268, 148, 104, 104], zone: null },
  };

  // Группы вентиляторов: 1–3 штуки по центру своего места (сверху шаг 150 по x, спереди 140 по y)
  const GROUPS: Record<string, { center: number; step: number }> = { topFans: { center: 395, step: 150 }, frontFans: { center: 290, step: 140 } };
  const fanSpots = (name: string) => {
    const n = app.fanCounts[name] ?? 3;
    const { center, step } = GROUPS[name];
    return Array.from({ length: n }, (_, i) => center + (i - (n - 1) / 2) * step);
  };
  function boxOf(name: string): Box {
    if (!(name in GROUPS)) return PARTS[name].box!;
    const spots = fanSpots(name);
    return name === 'topFans' ? [spots[0] - 72, 36, spots.at(-1)! - spots[0] + 144, 34] : [614, spots[0] - 64, 132, spots.at(-1)! - spots[0] + 128];
  }

  const partZone = (name: string): PartZone => (name in app.scene ? app.scene[name] : PARTS[name].zone);
  const zoneById = (id: number) => app.zones.find((z) => z.id === id);
  const partLook = (name: string) => {
    const z = partZone(name);
    return typeof z === 'number' ? zoneById(z) : undefined;
  };
  function zoneText(z: PartZone) {
    if (z === 'hidden') return 'нет на моём ПК';
    if (z === null) return 'без подсветки';
    const zone = zoneById(z);
    return zone ? zoneLabel(zone) : `зона ${z}`;
  }

  let editing = $state(false); // режим «Настроить»: клик по детали выбирает её зону
  let hover = $state<string | null>(null);
  let menu = $state<{ part: string; x: number; y: number } | null>(null);
  let view = { k: 1, ox: 0, oy: 0, dpr: 1 }; // как сцена легла на холст — для перевода координат мыши

  let host: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  const glow = document.createElement('canvas');
  let size = $state({ w: 0, h: 0 });

  $effect(() => {
    const ro = new ResizeObserver(([e]) => (size = { w: e.contentRect.width, h: e.contentRect.height }));
    ro.observe(host);
    return () => ro.disconnect();
  });

  const info = $derived.by(() => {
    const p = placeholders();
    return [...p.gpu, ...p.ram].find((d) => d.key === app.selected);
  });

  function roundRect(c: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number, fill: string, stroke?: string) {
    c.beginPath();
    c.roundRect(x, y, w, h, r);
    c.fillStyle = fill;
    c.fill();
    if (stroke) {
      c.strokeStyle = stroke;
      c.lineWidth = 1.5;
      c.stroke();
    }
  }

  /** Кольцо светодиодов: эллипс (rx, ry) из N сегментов, цвет каждого — из эффекта */
  function ring(g: CanvasRenderingContext2D, x: number, y: number, rx: number, ry: number, width: number, look: Look, t: number) {
    const N = 40;
    g.lineWidth = width;
    for (let i = 0; i < N; i++) {
      const a = (i / N) * Math.PI * 2 - Math.PI / 2;
      g.beginPath();
      g.ellipse(x, y, rx, ry, 0, a, a + (Math.PI * 2) / N + 0.01);
      g.strokeStyle = css(led(look, i / N, t));
      g.stroke();
    }
  }

  function average(look: Look, t: number): RGB {
    const s = [0, 0, 0];
    for (let i = 0; i < 8; i++) led(look, i / 8, t).forEach((v, k) => (s[k] += v / 8));
    return s as RGB;
  }

  function fan(c: CanvasRenderingContext2D, g: CanvasRenderingContext2D, x: number, y: number, r: number, look: Look | undefined, t: number) {
    roundRect(c, x - r * 1.08, y - r * 1.08, r * 2.16, r * 2.16, r * 0.22, '#0f141c', '#222c3b');
    c.beginPath();
    c.arc(x, y, r * 0.98, 0, Math.PI * 2);
    c.fillStyle = '#06080c';
    c.fill();
    const avg = look ? average(look, t) : ([0, 0, 0] as RGB);
    // лопасти чуть подсвечены кольцом и крутятся
    c.strokeStyle = css(mix([30, 36, 46], avg, 0.25));
    c.lineWidth = r * 0.1;
    c.lineCap = 'round';
    for (let k = 0; k < 7; k++) {
      const a = (k / 7) * Math.PI * 2 + t * 3;
      const at = (da: number, rr: number) => [x + Math.cos(a + da) * r * rr, y + Math.sin(a + da) * r * rr] as const;
      c.beginPath();
      c.moveTo(...at(0, 0.24));
      c.quadraticCurveTo(...at(0.4, 0.55), ...at(1, 0.72));
      c.stroke();
    }
    c.beginPath();
    c.arc(x, y, r * 0.24, 0, Math.PI * 2);
    c.fillStyle = '#151b25';
    c.fill();
    if (look && look.mode) ring(g, x, y, r * 0.86, r * 0.86, r * 0.09, look, t);
    else ring(c, x, y, r * 0.86, r * 0.86, r * 0.09, { mode: 0, color: [0, 0, 0], bright: 0, speed: 0, palette: [] }, t);
  }

  function strip(g: CanvasRenderingContext2D, [x0, y0, x1, y1]: number[], look: Look, t: number) {
    const N = 30;
    g.lineWidth = 5;
    g.lineCap = 'round';
    for (let i = 0; i < N; i++) {
      g.beginPath();
      g.moveTo(x0 + ((x1 - x0) * i) / N, y0 + ((y1 - y0) * i) / N);
      g.lineTo(x0 + ((x1 - x0) * (i + 1)) / N, y0 + ((y1 - y0) * (i + 1)) / N);
      g.strokeStyle = css(led(look, i / N, t));
      g.stroke();
    }
  }

  function scene(c: CanvasRenderingContext2D, g: CanvasRenderingContext2D, t: number) {
    // корпус и стекло
    const body = c.createLinearGradient(0, 0, W, H);
    body.addColorStop(0, '#0e131c');
    body.addColorStop(1, '#080b11');
    roundRect(c, 30, 14, 740, 532, 18, '#000', '#1f2a3a');
    c.fillStyle = body;
    c.fill();
    roundRect(c, 40, 492, 720, 46, 8, '#0b1017', '#1a2331');

    // материнская плата, процессорный кулер, память, видеокарта — без подсветки
    roundRect(c, 170, 100, 410, 300, 6, '#0f151e', '#212b3a');
    for (let i = 0; i < 6; i++) roundRect(c, 196 + i * 12, 240, 4, 130, 2, '#151c27');
    for (let i = 0; i < 4; i++) {
      roundRect(c, 420 + i * 22, 132, 12, 160, 3, '#171e29', '#232d3d');
      roundRect(c, 420 + i * 22, 132, 12, 10, 3, '#2a3343');
    }
    roundRect(c, 160, 420, 440, 54, 8, '#121821', '#232d3d');
    c.fillStyle = '#4c5668';
    c.font = '600 17px "Segoe UI Variable Text", "Segoe UI", sans-serif';
    c.fillText((app.sys.gpu?.[0]?.Name ?? 'GPU').toUpperCase(), 186, 453);

    // детали с подсветкой
    for (const name of Object.keys(PARTS)) {
      const l = partLook(name);
      for (const seg of PARTS[name].strips ?? []) if (l?.mode) strip(g, seg, l, t);
    }
    const shown = (name: string) => partZone(name) !== 'hidden';
    if (shown('cpu')) {
      const l = partLook('cpu');
      roundRect(c, 268, 148, 104, 104, 10, '#121821', '#263041');
      if (l?.mode) ring(g, 320, 200, 38, 38, 6, l, t);
      else {
        c.beginPath();
        c.arc(320, 200, 38, 0, Math.PI * 2);
        c.strokeStyle = '#273142';
        c.lineWidth = 6;
        c.stroke();
      }
    }
    if (shown('topFans')) {
      const l = partLook('topFans');
      for (const x of fanSpots('topFans')) {
        roundRect(c, x - 70, 36, 140, 32, 8, '#0f141c', '#222c3b');
        if (l?.mode) ring(g, x, 52, 58, 10, 5, l, t);
      }
    }
    if (shown('frontFans')) for (const y of fanSpots('frontFans')) fan(c, g, 680, y, 58, partLook('frontFans'), t);
    if (shown('rearFan')) fan(c, g, 105, 150, 44, partLook('rearFan'), t);
  }

  /** Рамки: детали выбранной зоны, деталь под мышью, в режиме «Настроить» — все */
  function outlines(c: CanvasRenderingContext2D) {
    for (const name of Object.keys(PARTS)) {
      const box = boxOf(name);
      const z = partZone(name);
      const picked = typeof z === 'number' && app.selected === `zone:${z}`;
      if (!editing && !picked && hover !== name) continue;
      c.beginPath();
      c.roundRect(box[0] - 5, box[1] - 5, box[2] + 10, box[3] + 10, 8);
      c.setLineDash(editing && hover !== name ? [6, 5] : []);
      c.lineWidth = 2;
      c.strokeStyle = hover === name ? 'rgba(255,255,255,0.75)' : picked ? '#2f7cf6' : 'rgba(90,160,255,0.45)';
      c.stroke();
    }
    c.setLineDash([]);
  }

  /** Деталь под указателем; вне «Настроить» — только та, что светится какой-то зоной */
  function partAt(e: MouseEvent) {
    const r = canvas.getBoundingClientRect();
    const x = ((e.clientX - r.left) * view.dpr - view.ox) / view.k;
    const y = ((e.clientY - r.top) * view.dpr - view.oy) / view.k;
    const name = Object.keys(PARTS).find((name) => {
      const [bx, by, bw, bh] = boxOf(name);
      return x >= bx - 5 && x <= bx + bw + 5 && y >= by - 5 && y <= by + bh + 5 && (editing || !!partLook(name)); // зона должна быть у контроллера, а не только номером в настройке
    });
    return name ?? null;
  }

  function click(e: MouseEvent) {
    const name = partAt(e);
    menu = null;
    if (!name) return;
    if (!editing) return select(`zone:${partZone(name)}`);
    // меню выбора зоны у курсора, но не за краем картинки
    const r = host.getBoundingClientRect();
    const height = (app.zones.length + 3) * 34 + (name in GROUPS ? 44 : 0) + 16;
    menu = { part: name, x: Math.min(e.clientX - r.left, r.width - 230), y: Math.max(8, Math.min(e.clientY - r.top, r.height - height)) };
  }

  function pick(z: PartZone) {
    if (menu) setPartZone(menu.part, z);
    menu = null;
  }

  $effect(() => {
    const t = app.now;
    const { w, h } = size;
    if (!w || !h) return;
    const dpr = devicePixelRatio;
    const [pw, ph] = [Math.round(w * dpr), Math.round(h * dpr)];
    if (canvas.width !== pw || canvas.height !== ph) {
      [canvas.width, canvas.height, glow.width, glow.height] = [pw, ph, pw, ph];
    }
    const c = canvas.getContext('2d')!;
    const g = glow.getContext('2d')!;
    const k = Math.min(pw / W, ph / H);
    view = { k, ox: (pw - W * k) / 2, oy: (ph - H * k) / 2, dpr };
    for (const ctx of [c, g]) {
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.clearRect(0, 0, pw, ph);
      ctx.setTransform(k, 0, 0, k, view.ox, view.oy);
    }
    scene(c, g, t);
    // свечение: размытый слой светодиодов под чётким
    c.setTransform(1, 0, 0, 1, 0, 0);
    c.globalCompositeOperation = 'lighter';
    c.filter = `blur(${14 * k}px)`;
    c.drawImage(glow, 0, 0);
    c.filter = 'none';
    c.globalCompositeOperation = 'source-over';
    c.drawImage(glow, 0, 0);
    c.setTransform(k, 0, 0, k, view.ox, view.oy);
    outlines(c);
  });
</script>

<div class="preview" bind:this={host}>
  <!-- клик по детали дублирует выбор зоны в дереве слева, поэтому без отдельного доступа с клавиатуры -->
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <canvas
    bind:this={canvas}
    class:pointer={hover}
    onpointermove={(e) => (hover = partAt(e))}
    onpointerleave={() => (hover = null)}
    onclick={click}
  ></canvas>

  <button class="edit" class:on={editing} onclick={() => ([editing, menu] = [!editing, null])}>
    {#if editing}<Check size={14} />Готово{:else}<Pencil size={14} />Настроить{/if}
  </button>

  {#if menu}
    {@const current = partZone(menu.part)}
    <div class="menu" style:left="{menu.x}px" style:top="{menu.y}px" role="menu">
      <b>{PARTS[menu.part].label}</b>
      {#if menu.part in GROUPS}
        {@const count = fanSpots(menu.part).length}
        <div class="count" role="group" aria-label="Сколько вентиляторов">
          <span>Вентиляторов</span>
          {#each [1, 2, 3] as n (n)}
            <button class:on={count === n} aria-pressed={count === n} onclick={() => setFanCount(menu!.part, n)}>{n}</button>
          {/each}
        </div>
      {/if}
      {#each app.zones as z (z.id)}
        <button role="menuitemradio" aria-checked={current === z.id} onclick={() => pick(z.id)}>{zoneLabel(z)}{#if current === z.id}<Check size={14} />{/if}</button>
      {/each}
      <hr />
      <button role="menuitemradio" aria-checked={current === null} onclick={() => pick(null)}>Без подсветки{#if current === null}<Check size={14} />{/if}</button>
      <button role="menuitemradio" aria-checked={current === 'hidden'} onclick={() => pick('hidden')}>Нет на моём ПК{#if current === 'hidden'}<Check size={14} />{/if}</button>
    </div>
  {/if}

  {#if !info && (editing || hover)}
    <p class="hint">
      {#if hover}<b>{PARTS[hover].label}</b> · {zoneText(partZone(hover))}{:else}Нажмите на деталь и выберите, какой зоной она светится{/if}
    </p>
  {/if}
  {#if info}
    <div class="info">
      <b>{info.name}</b>
      <dl>
        {#each info.info as [k, v] (k)}<dt>{k}</dt><dd>{v}</dd>{/each}
      </dl>
      <p>Управление подсветкой этого устройства пока не поддерживается.</p>
    </div>
  {/if}
</div>

<style>
  .preview {
    position: relative;
    min-height: 0;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: radial-gradient(ellipse at 50% 40%, #111a29, #06090f 75%);
    overflow: hidden;
  }
  canvas { position: absolute; inset: 0; width: 100%; height: 100%; }
  canvas.pointer { cursor: pointer; }
  .edit {
    position: absolute;
    top: 12px;
    right: 12px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    background: rgba(14, 20, 30, 0.85);
    color: var(--dim);
    font: inherit;
    font-size: 12px;
    font-weight: 500;
  }
  .edit:hover, .edit.on { color: var(--text); }
  .edit.on { border-color: var(--accent); }
  .hint {
    position: absolute;
    left: 16px;
    bottom: 14px;
    margin: 0;
    padding: 6px 10px;
    border-radius: 8px;
    background: rgba(14, 20, 30, 0.85);
    color: var(--dim);
    font-size: 12px;
    pointer-events: none;
  }
  .hint b { color: var(--text); font-weight: 500; }
  .menu {
    position: absolute;
    z-index: 2;
    width: 220px;
    padding: 4px;
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    background: var(--card-hover);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  }
  .menu b { display: block; padding: 6px 10px 4px; color: var(--dim); font-size: 12px; font-weight: 500; }
  .menu button {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 7px 10px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: 13px;
    text-align: left;
  }
  .menu button:hover, .menu button:focus-visible { background: rgba(255, 255, 255, 0.08); }
  .menu :global(svg) { color: var(--accent); }
  hr { margin: 4px 6px; border: 0; border-top: 1px solid var(--border); }
  .count { display: flex; align-items: center; gap: 4px; padding: 2px 6px 6px; }
  .menu .count button { justify-content: center; width: 32px; padding: 5px 0; border: 1px solid var(--border-strong); }
  .menu .count button.on { border-color: var(--accent); background: var(--accent-soft); }
  .count span { flex: 1; padding-left: 4px; color: var(--dim); font-size: 12px; }
  .info {
    position: absolute;
    left: 16px;
    bottom: 16px;
    max-width: 320px;
    padding: 14px 16px;
    border-radius: 10px;
    background: rgba(14, 20, 30, 0.92);
    border: 1px solid var(--border-strong);
    backdrop-filter: blur(6px);
  }
  dl { display: grid; grid-template-columns: auto 1fr; gap: 4px 14px; margin: 10px 0; font-size: 13px; }
  dt { color: var(--dim); }
  dd { margin: 0; }
  p { margin: 0; color: var(--dim); font-size: 12px; }
</style>
