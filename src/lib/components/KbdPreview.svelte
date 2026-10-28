<script lang="ts">
  import { app } from '$lib/state/app.svelte';
  import { FULL, BRIGHTS, drawKeys } from '$lib/kbd';

  // Клавиатура 22,5 × 6,5 клавиш (с промежутком под F-рядом) в корпусе с полями PAD
  const KW = 22.5, KH = 6.5, PAD = 0.6;

  let host: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  const glow = document.createElement('canvas');
  let size = $state({ w: 0, h: 0 });

  $effect(() => {
    const ro = new ResizeObserver(([e]) => (size = { w: e.contentRect.width, h: e.contentRect.height }));
    ro.observe(host);
    return () => ro.disconnect();
  });

  const e = $derived(app.kbd!.effect);
  const bright = $derived((BRIGHTS[e.brightness] ?? 100) / 100); // за таблицей — «динамическая», рисуем полной

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
    c.clearRect(0, 0, pw, ph);
    g.clearRect(0, 0, pw, ph);

    const u = Math.min((pw * 0.92) / (KW + PAD * 2), (ph * 0.8) / (KH + PAD * 2));
    const x0 = (pw - KW * u) / 2;
    const y0 = (ph - KH * u) / 2;
    c.beginPath();
    c.roundRect(x0 - PAD * u, y0 - PAD * u, (KW + PAD * 2) * u, (KH + PAD * 2) * u, u * 0.4);
    c.fillStyle = '#0b0f17';
    c.fill();
    c.strokeStyle = '#1f2836';
    c.lineWidth = 1.5 * dpr;
    c.stroke();

    drawKeys(g, FULL, e, t, x0, y0, u, u, 0.5, bright);
    // свечение: размытый слой клавиш под чётким, как у сцены компьютера
    c.globalCompositeOperation = 'lighter';
    c.filter = `blur(${u * 0.5}px)`;
    c.drawImage(glow, 0, 0);
    c.filter = 'none';
    c.globalCompositeOperation = 'source-over';
    c.drawImage(glow, 0, 0);
  });
</script>

<div class="preview" bind:this={host}>
  <canvas bind:this={canvas}></canvas>
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
</style>
