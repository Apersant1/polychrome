<script lang="ts">
  import { app } from '$lib/state/app.svelte';
  import { GRID, drawKeys, type Shown } from '$lib/kbd';

  /** Превью режима клавиатуры на сетке клавиш */
  let { shown }: { shown: Shown } = $props();
  let canvas: HTMLCanvasElement;

  $effect(() => {
    const t = app.now;
    const c = canvas.getContext('2d')!;
    const { width: w, height: h } = canvas;
    c.clearRect(0, 0, w, h);
    drawKeys(c, GRID, shown, t, 16, 6, (w - 32) / 15, (h - 12) / 4);
  });
</script>

<canvas bind:this={canvas} width="340" height="70"></canvas>

<style>
  canvas { display: block; width: 100%; height: 38px; border-radius: 6px; background: #080c13; }
</style>
