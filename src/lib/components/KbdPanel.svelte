<script lang="ts">
  import Info from '@lucide/svelte/icons/info';
  import Sun from '@lucide/svelte/icons/sun';
  import Gauge from '@lucide/svelte/icons/gauge';
  import Rainbow from '@lucide/svelte/icons/rainbow';
  import Dices from '@lucide/svelte/icons/dices';
  import ArrowRight from '@lucide/svelte/icons/arrow-right';
  import ArrowLeft from '@lucide/svelte/icons/arrow-left';
  import ArrowDown from '@lucide/svelte/icons/arrow-down';
  import ArrowUp from '@lucide/svelte/icons/arrow-up';
  import ColorPicker from './ColorPicker.svelte';
  import { app } from '$lib/state/app.svelte';
  import { kbdSet, KBD_NAMES } from '$lib/state/keyboard.svelte';
  import { COLOR_FLAGS, BRIGHTS, HUE, RANDOM, rgb, withRgb } from '$lib/kbd';
  import { css, hex, type RGB } from '$lib/leds';

  const DIRS = [['Вправо', ArrowRight], ['Влево', ArrowLeft], ['Вниз', ArrowDown], ['Вверх', ArrowUp]] as const;
  const CUSTOM = 13;
  let sel = $state(0); // какой цвет режима правим

  const kbd = $derived(app.kbd!);
  const e = $derived(kbd.effect);
  const info = $derived(kbd.modes.find((m) => m.index === e.mode));
  const colors = $derived(Array.from({ length: info?.colors ?? 0 }, (_, i) => e.palette[i] ?? 0));
  // У двухцветных режимов первый цвет — фон, у тепловой карты — от редких нажатий к частым
  const NAMES = $derived(e.mode === 12 ? ['Редко', 'Часто'] : ['Фон', 'Эффект']);
  const flag = $derived(COLOR_FLAGS[e.mode]?.[sel] ?? 0); // какой флаг можно включить выбранному цвету
  const flagOn = (c: number) => (c & (HUE | RANDOM)) !== 0;

  $effect(() => {
    if (sel >= colors.length) sel = 0;
  });

  const set = (patch: Partial<typeof e>) => kbdSet({ ...e, ...patch });

  const setColor = (c: RGB) => set({ palette: colors.map((p, i) => (i === sel ? withRgb(p, c) : p)) });
  const toggleFlag = () => set({ palette: colors.map((p, i) => (i === sel ? (flagOn(p) ? p & 0xffffff : p | flag) : p)) });
</script>

{#snippet slider(id: string, label: string, Icon: typeof Sun, value: number, max: number, key: 'brightness' | 'speed', shown: string)}
  <div class="row">
    <label for={id}><Icon size={16} />{label}</label>
    <input {id} type="range" min="0" {max} {value} style:--fill="{(value / max) * 100}%" oninput={(ev) => set({ [key]: +ev.currentTarget.value })} />
    <b>{shown}</b>
  </div>
{/snippet}

{#snippet tip(id: string, text: string)}
  <button type="button" class="tip" aria-label="Подсказка" aria-describedby={id}><Info size={14} /></button>
  <span {id} role="tooltip">{text}</span>
{/snippet}

<div class="panel kbd">
  <h3>
    Клавиатура<span>· ZET Blade Pro</span>
    {@render tip('kbd-tip', 'Настройки хранит сама клавиатура. Blade Pro Software не нужна, а если запущена — занимает клавиатуру.')}
  </h3>

  {#if app.kbdFollow}
    <p class="mode">Режим: <b>Как у платы</b></p>
    <p class="note">Клавиатура светится статичным цветом выбранной зоны и меняется вместе с ней. Чтобы настроить клавиатуру отдельно, выберите другой режим.</p>
  {:else}
    <p class="mode">Режим: <b>{KBD_NAMES[e.mode] ?? `Режим ${e.mode}`}</b></p>

    {#if colors.length}
      <div class="palette">
        {#if colors.length > 1}
          {#each colors as c, i (i)}
            <button class="chip" class:on={i === sel} aria-pressed={i === sel} aria-label="{NAMES[i]}: {flagOn(c) ? 'меняется сам' : hex(rgb(c))}" onclick={() => (sel = i)}>
              <i class:auto={flagOn(c)} style:background={flagOn(c) ? null : css(rgb(c))}></i>{NAMES[i]}
            </button>
          {/each}
        {/if}
        {#if flag}
          <button class="flag" class:on={flagOn(colors[sel])} aria-pressed={flagOn(colors[sel])} onclick={toggleFlag}>
            {#if flag === HUE}<Rainbow size={16} />Радуга{:else}<Dices size={16} />Случайный{/if}
          </button>
        {/if}
      </div>

      {#if flagOn(colors[sel])}
        <p class="note">{flag === HUE ? 'Оттенки перебирает сама клавиатура.' : 'Клавиатура каждый раз берёт случайный цвет.'}</p>
      {:else}
        <ColorPicker color={rgb(colors[sel] ?? 0)} set={setColor} />
      {/if}
    {/if}

    {#if info && info.brights > 1}
      {@render slider('kbd-bright', 'Яркость', Sun, e.brightness, info.brights - 1, 'brightness', e.brightness in BRIGHTS ? `${BRIGHTS[e.brightness]}%` : 'Авто')}
    {/if}
    <!-- у статичного скорость есть только у радуги: как быстро меняется оттенок -->
    {#if info && info.speeds > 1 && (e.mode !== 1 || flagOn(colors[0] ?? 0))}
      {@render slider('kbd-speed', 'Скорость', Gauge, e.speed, info.speeds - 1, 'speed', `${Math.round(((e.speed + 1) / info.speeds) * 100)}%`)}
    {/if}

    {#if info && info.dirs > 0}
      <div class="dirs" role="radiogroup" aria-label="Направление">
        {#each DIRS.slice(0, info.dirs) as [name, Icon], i (name)}
          <button role="radio" aria-checked={e.direction === i} class:on={e.direction === i} onclick={() => set({ direction: i })}><Icon size={14} />{name}</button>
        {/each}
      </div>
    {/if}

    {#if e.mode === CUSTOM}<p class="note">Цвета отдельных клавиш остаются те, что записаны в клавиатуре.</p>{/if}
  {/if}
</div>

<style>
  .kbd { display: flex; flex-direction: column; gap: 14px; overflow: auto; }
  h3 { position: relative; display: flex; align-items: center; margin-bottom: 0; }
  h3 span { margin: 0 6px; color: var(--dim); font-weight: 400; }
  .tip { display: grid; place-items: center; padding: 2px; border: 0; border-radius: 50%; background: none; color: var(--dim); cursor: help; }
  .tip:hover, .tip:focus-visible { color: var(--text); }
  /* подсказка во всю ширину заголовка, открывается вниз */
  [role='tooltip'] {
    position: absolute;
    left: 0;
    right: 0;
    z-index: 1;
    display: none;
    padding: 8px 10px;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    background: var(--card);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.5);
    color: var(--text);
    font-size: 12px;
    font-weight: 400;
    line-height: 1.5;
  }
  .tip:hover + [role='tooltip'], .tip:focus-visible + [role='tooltip'] { display: block; }
  [role='tooltip'] { top: calc(100% + 6px); }
  .mode { margin: 0; color: var(--dim); font-size: 13px; }
  .mode b { color: var(--text); font-weight: 500; }
  .palette { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; }
  .chip i { width: 18px; height: 18px; border-radius: 5px; border: 1px solid rgba(255, 255, 255, 0.2); }
  .chip i.auto { background: conic-gradient(#f44, #fd4, #4f8, #4cf, #84f, #f4c, #f44); }
  .chip, .flag, .dirs button { display: flex; align-items: center; gap: 6px; padding: 6px 10px; border: 1px solid var(--border-strong); border-radius: 8px; background: var(--card); color: var(--dim); font-size: 13px; }
  .chip.on, .flag.on, .dirs button.on { border-color: var(--accent); color: var(--text); }
  .flag.on :global(svg) { color: var(--accent); }
  .dirs { display: flex; flex-wrap: wrap; gap: 6px; }
  .row { display: flex; align-items: center; gap: 12px; }
  .row label { display: flex; align-items: center; gap: 8px; flex: none; width: 92px; color: var(--dim); font-size: 13px; }
  .row b { width: 40px; text-align: right; color: var(--text); font-weight: 500; }
  .note { margin: 0; color: var(--dim); font-size: 12px; line-height: 1.5; }
</style>
