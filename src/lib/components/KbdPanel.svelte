<script lang="ts">
  import { app } from '$lib/state/app.svelte';
  import { kbdSet, setKbdFollow, KBD_NAMES } from '$lib/state/keyboard.svelte';

  const DIRS = ['Вправо', 'Влево', 'Вниз', 'Вверх'];
  const HUE = 0x1000000; // флаг цвета: прошивка сама перебирает оттенки
  const CUSTOM = 13;

  const kbd = $derived(app.kbd!);
  const e = $derived(kbd.effect);
  const info = $derived(kbd.modes.find((m) => m.index === e.mode));
  const colors = $derived(Array.from({ length: info?.colors ?? 0 }, (_, i) => e.palette[i] ?? 0));
  const rainbow = $derived(colors.length > 0 && colors.every((c) => c & HUE));

  const hex = (c: number) => '#' + (c & 0xffffff).toString(16).padStart(6, '0');
  const set = (patch: Partial<typeof e>) => kbdSet({ ...e, ...patch });

  function setColor(i: number, value: string) {
    const palette = [...colors];
    palette[i] = parseInt(value.slice(1), 16) | (palette[i] & HUE);
    set({ palette });
  }

  const setRainbow = (on: boolean) => set({ palette: colors.map((c) => (on ? c | HUE : c & ~HUE)) });
</script>

{#snippet slider(id: string, label: string, value: number, max: number, key: 'brightness' | 'speed')}
  <div class="row">
    <label for={id}>{label}</label>
    <input {id} type="range" min="0" {max} {value} style:--fill="{max ? (value / max) * 100 : 100}%" oninput={(ev) => set({ [key]: +ev.currentTarget.value })} />
    <b>{value + 1}/{max + 1}</b>
  </div>
{/snippet}

<div class="panel kbd">
  <h3>Клавиатура<span>· ZET Blade Pro</span></h3>

  <p class="mode">Режим: <b>{KBD_NAMES[e.mode] ?? `Режим ${e.mode}`}</b></p>

  {#if colors.length}
    <div class="field">
      <span>Цвет{colors.length > 1 ? 'а' : ''}</span>
      <div class="colors">
        {#each colors as c, i (i)}
          <input type="color" value={hex(c)} disabled={rainbow} aria-label="Цвет {i + 1}" oninput={(ev) => setColor(i, ev.currentTarget.value)} />
        {/each}
        <label class="check"><input type="checkbox" checked={rainbow} onchange={(ev) => setRainbow(ev.currentTarget.checked)} />Радуга</label>
      </div>
    </div>
  {/if}

  {#if info && info.brights > 1}{@render slider('kbd-bright', 'Яркость', e.brightness, info.brights - 1, 'brightness')}{/if}
  {#if info && info.speeds > 1 && e.mode !== 1}{@render slider('kbd-speed', 'Скорость', e.speed, info.speeds - 1, 'speed')}{/if}

  {#if info && info.dirs > 0}
    <label class="field">
      <span>Направление</span>
      <select value={e.direction} onchange={(ev) => set({ direction: +ev.currentTarget.value })}>
        {#each DIRS.slice(0, info.dirs) as d, i (d)}<option value={i}>{d}</option>{/each}
      </select>
    </label>
  {/if}

  {#if e.mode === CUSTOM}<p class="note">Цвета отдельных клавиш остаются те, что записаны в клавиатуре.</p>{/if}

  <label class="follow">
    <span>Цвет как у платы<small>правки зон красят клавиатуру статичным цветом</small></span>
    <input type="checkbox" role="switch" class="switch" checked={app.kbdFollow} onchange={(ev) => setKbdFollow(ev.currentTarget.checked)} />
  </label>
  <p class="note">Настройки хранит сама клавиатура. Программа Blade Pro Software не нужна, а если запущена — занимает клавиатуру.</p>
</div>

<style>
  .kbd { display: flex; flex-direction: column; gap: 14px; overflow: auto; }
  h3 { margin-bottom: 0; }
  h3 span { margin-left: 6px; color: var(--dim); font-weight: 400; }
  .mode { margin: 0; color: var(--dim); font-size: 13px; }
  .mode b { color: var(--text); font-weight: 500; }
  .field { display: grid; gap: 6px; font-size: 13px; }
  .field > span { color: var(--dim); }
  .colors { display: flex; align-items: center; flex-wrap: wrap; gap: 10px; }
  input[type='color'] { width: 40px; height: 30px; padding: 0; border: 1px solid var(--border-strong); border-radius: 8px; background: none; cursor: pointer; }
  input[type='color']:disabled { opacity: 0.35; cursor: default; }
  .check { display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; }
  .row { display: grid; grid-template-columns: 90px 1fr 40px; align-items: center; gap: 10px; font-size: 13px; }
  .row label { color: var(--dim); }
  .row b { font-weight: 500; text-align: right; font-size: 12px; }
  .follow { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-top: auto; padding-top: 12px; border-top: 1px solid var(--border); cursor: pointer; }
  .follow span { display: grid; gap: 2px; font-size: 13px; }
  small { color: var(--dim); font-size: 12px; }
  .note { margin: 0; color: var(--dim); font-size: 12px; line-height: 1.5; }
</style>
