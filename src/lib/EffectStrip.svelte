<script lang="ts">
  import { app } from './app.svelte';
  import { led, css, type Look } from './leds';

  /** wave — насколько линия изгибается (0 — прямая) */
  let { look, wave = 0 }: { look: Look; wave?: number } = $props();
  let canvas: HTMLCanvasElement;

  $effect(() => {
    const t = app.now;
    const c = canvas.getContext('2d')!;
    const { width: w, height: h } = canvas;
    c.clearRect(0, 0, w, h);
    c.lineCap = 'round';
    const N = 40;
    const y = (x: number) => h / 2 + wave * h * 0.3 * Math.sin(Math.PI * 2 * (x * 1.5 - t * 0.4));
    for (const [width, alpha] of [[16, 0.12], [8, 0.3], [4, 1]]) {
      c.lineWidth = width;
      for (let i = 0; i < N; i++) {
        const [x0, x1] = [i / N, (i + 1) / N];
        c.beginPath();
        c.moveTo(16 + x0 * (w - 32), y(x0));
        c.lineTo(16 + x1 * (w - 32), y(x1));
        c.strokeStyle = css(led(look, x0, t), alpha);
        c.stroke();
      }
    }
  });
</script>

<canvas bind:this={canvas} width="340" height="70"></canvas>

<style>
  canvas { display: block; width: 100%; height: 38px; border-radius: 6px; background: #080c13; }
</style>
