<script lang="ts">
  import { app } from '$lib/state/app.svelte';
  import { css, hsv, scale, type RGB } from '$lib/leds';

  /** Превью режима клавиатуры на сетке клавиш; color null — прошивка сама перебирает оттенки */
  let { mode, color }: { mode: number; color: RGB | null } = $props();
  let canvas: HTMLCanvasElement;

  const COLS = 15;
  const ROWS = 4;
  const DARK: RGB = [18, 20, 26];
  const rand = (a: number) => {
    const s = Math.sin(a * 12.9898) * 43758.5453;
    return s - Math.floor(s);
  };
  const frac = (v: number) => v - Math.floor(v);
  const clamp = (v: number) => Math.min(Math.max(v, 0), 1);

  /** Нажатия, придуманные для превью: каждые 0,4 с случайная клавиша; [столбец, строка, сколько секунд назад] */
  function presses(t: number, n: number, every = 0.4) {
    const now = Math.floor(t / every);
    return Array.from({ length: n }, (_, i) => {
      const s = now - i;
      return [Math.floor(rand(s) * COLS), Math.floor(rand(s + 0.5) * ROWS), t - s * every] as const;
    });
  }

  /** Цвет клавиши (x — столбец, y — строка) в момент t; яркость отдельно, чтобы не гасить клавишу ниже фона */
  function key(x: number, y: number, t: number): [RGB, number] {
    const c = color ?? hsv(frac(x / COLS - t * 0.2), 1, 1);
    const id = y * COLS + x;
    switch (mode) {
      case 1: return [c, 1];
      case 2: return [c, 0.5 - 0.5 * Math.cos(t * 2.1)];
      case 3: return [hsv(frac(x / COLS - t * 0.3), 1, 1), 1];
      case 4: { // капля падает по своему столбцу, хвост выше неё
        const head = frac(t * 0.7 + rand(x)) * (ROWS + 2) - 1;
        return [c, head >= y ? clamp(1 - (head - y) / 1.5) : 0];
      }
      case 5: { // огонь бежит змейкой по рядам
        const pos = y % 2 ? COLS - 1 - x : x;
        const head = frac(t * 0.25) * COLS * ROWS;
        const d = (head - (y * COLS + pos) + COLS * ROWS) % (COLS * ROWS);
        return [hsv(0.08 * clamp(1 - d / 8), 1, 1), clamp(1 - d / 8)];
      }
      case 6: return [c, Math.max(0, Math.sin(2 * Math.PI * (t * (0.3 + rand(id) * 0.4) + rand(id + 7)))) ** 8];
      case 7: return [c, Math.max(0, ...presses(t, 3).map(([px, py, age]) => (px === x && py === y ? 1 - age / 1.2 : 0)))];
      case 8: case 10: { // кольца от нажатий; рябь радужная
        let k = 0;
        for (const [px, py, age] of presses(t, 2, 0.9)) {
          const d = Math.hypot(x - px, (y - py) * 1.2);
          k = Math.max(k, clamp(1 - Math.abs(d - age * 7) / 1.2) * clamp(1 - age / 1.8));
        }
        return [mode === 10 ? hsv(frac((x + y) / 12 - t * 0.3), 1, 1) : c, k];
      }
      case 9: { // крест: ряд и столбец нажатой клавиши расходятся от неё
        let k = 0;
        for (const [px, py, age] of presses(t, 2, 0.9)) {
          const d = x === px ? Math.abs(y - py) : y === py ? Math.abs(x - px) : Infinity;
          k = Math.max(k, clamp(1 - Math.abs(d - age * 8) / 1.5) * clamp(1 - age / 1.8));
        }
        return [c, k];
      }
      case 11: return [hsv(0.42 + 0.22 * Math.sin(x * 0.35 + t * 0.6 + Math.sin(y * 0.8 + t * 0.4)), 0.9, 1), 0.7 + 0.3 * Math.sin(x * 0.5 - t)];
      case 12: { // теплее там, где «чаще жмут»: середина и левая часть
        const heat = clamp(0.9 - Math.hypot((x - 4) / 7, (y - 1.5) / 3) + 0.15 * Math.sin(t + rand(id) * 6));
        return [hsv(0.66 * (1 - heat), 1, 1), 1];
      }
      case 13: return [hsv(rand(id), 0.9, 1), 1];
      default: return [c, 1];
    }
  }

  $effect(() => {
    const t = app.now;
    const c = canvas.getContext('2d')!;
    const { width: w, height: h } = canvas;
    c.clearRect(0, 0, w, h);
    const kw = (w - 32) / COLS;
    const kh = (h - 12) / ROWS;
    for (let y = 0; y < ROWS; y++)
      for (let x = 0; x < COLS; x++) {
        const [rgb, k] = key(x, y, t);
        const lit = scale(rgb, k);
        c.fillStyle = css(lit.map((v, i) => Math.max(v, DARK[i])) as RGB);
        c.beginPath();
        c.roundRect(16 + x * kw + 1.5, 6 + y * kh + 1.5, kw - 3, kh - 3, 2);
        c.fill();
      }
  });
</script>

<canvas bind:this={canvas} width="340" height="70"></canvas>

<style>
  canvas { display: block; width: 100%; height: 38px; border-radius: 6px; background: #080c13; }
</style>
