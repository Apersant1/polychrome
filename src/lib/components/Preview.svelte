<script lang="ts">
  import { app } from '$lib/state/app.svelte';
  import { FANS_ZONE } from '$lib/state/board.svelte';
  import { placeholders } from '$lib/state/system.svelte';
  import { led, css, mix, type Look, type RGB } from '$lib/leds';

  // Сцена 800×560, растягивается по месту с сохранением пропорций
  const W = 800, H = 560;
  // Куда на картинке выведены зоны контроллера (кроме кулеров): отрезки лент [x0, y0, x1, y1]
  const STRIPS: Record<number, [number, number, number, number][]> = {
    0: [[44, 50, 44, 470]],
    1: [[70, 28, 740, 28]],
    3: [[50, 490, 750, 490]],
    4: [[758, 70, 758, 470]],
    5: [[182, 112, 182, 196], [182, 112, 236, 112]],
    6: [[490, 106, 574, 106], [574, 106, 574, 170]], // уголок платы справа сверху
    // 7 — разъём ARGB #3, на картинке ленты к нему нет
  };

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
    const zone = (id: number) => app.zones.find((z) => z.id === id);
    const fans = zone(FANS_ZONE);

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
    roundRect(c, 268, 148, 104, 104, 10, '#121821', '#263041');
    c.beginPath();
    c.arc(320, 200, 38, 0, Math.PI * 2);
    c.strokeStyle = '#273142';
    c.lineWidth = 6;
    c.stroke();
    for (let i = 0; i < 4; i++) {
      roundRect(c, 420 + i * 22, 132, 12, 160, 3, '#171e29', '#232d3d');
      roundRect(c, 420 + i * 22, 132, 12, 10, 3, '#2a3343');
    }
    roundRect(c, 160, 420, 440, 54, 8, '#121821', '#232d3d');
    c.fillStyle = '#4c5668';
    c.font = '600 17px "Segoe UI Variable Text", "Segoe UI", sans-serif';
    c.fillText((app.sys.gpu?.[0]?.Name ?? 'GPU').toUpperCase(), 186, 453);

    // зоны контроллера
    for (const [id, segs] of Object.entries(STRIPS)) {
      const z = zone(+id);
      if (z && z.mode) for (const s of segs) strip(g, s, z, t);
    }
    for (const x of [245, 395, 545]) {
      roundRect(c, x - 70, 36, 140, 32, 8, '#0f141c', '#222c3b');
      if (fans && fans.mode) ring(g, x, 52, 58, 10, 5, fans, t);
    }
    for (const y of [150, 290, 430]) fan(c, g, 680, y, 58, fans, t);
    fan(c, g, 105, 150, 44, fans, t);
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
    for (const ctx of [c, g]) {
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.clearRect(0, 0, pw, ph);
      ctx.setTransform(k, 0, 0, k, (pw - W * k) / 2, (ph - H * k) / 2);
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
  });
</script>

<div class="preview" bind:this={host}>
  <canvas bind:this={canvas}></canvas>
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
