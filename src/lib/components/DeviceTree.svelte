<script lang="ts">
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import CircuitBoard from '@lucide/svelte/icons/circuit-board';
  import MemoryStick from '@lucide/svelte/icons/memory-stick';
  import Gpu from '@lucide/svelte/icons/gpu';
  import Keyboard from '@lucide/svelte/icons/keyboard';
  import Pencil from '@lucide/svelte/icons/pencil';
  import { app, type Zone } from '$lib/state/app.svelte';
  import { select, setPower, zoneLabel, defaultLabel } from '$lib/state/board.svelte';
  import { renameZone } from '$lib/state/config.svelte';
  import { placeholders } from '$lib/state/system.svelte';
  import { css, actual, MODE_OFF } from '$lib/leds';

  const extra = $derived(placeholders());
  let closed = $state<Record<string, boolean>>({});
  let editing = $state<number>(); // зона, которую переименовываем

  /** Enter/blur — сохранить (пустое имя вернёт стандартное), Esc — отменить */
  function finish(id: number, value: string | null) {
    if (editing !== id) return;
    editing = undefined;
    if (value !== null) renameZone(id, value);
  }

  const lit = $derived(app.zones.filter((z) => z.mode !== MODE_OFF).length);

  const dot = (z: Zone) => (z.mode === MODE_OFF ? '#2a3140' : css(actual(z).rgb));
</script>

{#snippet zoneRow(z: Zone)}
  <li class:selected={app.selected === `zone:${z.id}`} class:off={z.mode === MODE_OFF}>
    {#if editing === z.id}
      <i style:background={dot(z)}></i>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="name"
        aria-label="Имя зоны"
        placeholder={defaultLabel(z)}
        value={app.names[z.id] ?? zoneLabel(z)}
        autofocus
        onfocus={(e) => e.currentTarget.select()}
        onkeydown={(e) => (e.key === 'Enter' ? finish(z.id, e.currentTarget.value) : e.key === 'Escape' && finish(z.id, null))}
        onblur={(e) => finish(z.id, e.currentTarget.value)}
      />
    {:else}
      <button
        onclick={() => select(`zone:${z.id}`)}
        ondblclick={() => (editing = z.id)}
        onkeydown={(e) => e.key === 'F2' && (editing = z.id)}
      >
        <i style:background={dot(z)} style:box-shadow="0 0 8px {dot(z)}"></i>{zoneLabel(z)}
      </button>
      <button class="rename" aria-label="Переименовать {zoneLabel(z)}" onclick={() => (editing = z.id)}><Pencil size={13} /></button>
    {/if}
    <input
      type="checkbox"
      role="switch"
      class="switch"
      checked={z.mode !== MODE_OFF}
      onchange={(e) => setPower([z.id], e.currentTarget.checked)}
      aria-label="Подсветка {zoneLabel(z)}"
    />
  </li>
{/snippet}

{#snippet group(id: string, title: string, Icon: typeof CircuitBoard, body: import('svelte').Snippet)}
  <section>
    <button class="head" onclick={() => (closed[id] = !closed[id])}>
      <span class="chev" class:closed={closed[id]}><ChevronDown size={14} /></span>
      <Icon size={17} />{title}
    </button>
    {#if !closed[id]}<ul>{@render body()}</ul>{/if}
  </section>
{/snippet}

{#snippet boardRows()}
  {#each app.zones as z (z.id)}{@render zoneRow(z)}{/each}
{/snippet}
{#snippet ramRows()}
  {#each extra.ram as d (d.key)}
    <li class="stub" class:selected={app.selected === d.key}>
      <button onclick={() => select(d.key)}>{d.name || 'Модуль памяти'}</button>
    </li>
  {/each}
{/snippet}
{#snippet kbdRows()}
  <li class:selected={app.selected === 'kbd'}>
    <button onclick={() => select('kbd')}><i style:background="#5aa0ff"></i>ZET Blade Pro</button>
  </li>
{/snippet}
{#snippet gpuRows()}
  {#each extra.gpu as d (d.key)}
    <li class="stub" class:selected={app.selected === d.key}>
      <button onclick={() => select(d.key)}>{d.name}</button>
    </li>
  {/each}
{/snippet}

<div class="panel tree">
  <h3>
    Устройства
    <!-- Пока включены не все, тумблер выключен и включает все (так советует Adobe Spectrum) -->
    <input
      type="checkbox"
      role="switch"
      class="switch"
      title="Подсветка всех зон: светятся {lit} из {app.zones.length}"
      aria-label="Подсветка всех зон"
      disabled={app.status !== 'ok'}
      checked={lit > 0 && lit === app.zones.length}
      onchange={() => setPower(app.zones.map((z) => z.id), lit !== app.zones.length)}
    />
  </h3>
  {@render group('board', app.sys.board?.Product ?? 'Материнская плата', CircuitBoard, boardRows)}
  {#if app.kbd}{@render group('kbd', 'Клавиатура', Keyboard, kbdRows)}{/if}
  {#if extra.ram.length}{@render group('ram', 'Память', MemoryStick, ramRows)}{/if}
  {#if extra.gpu.length}{@render group('gpu', 'Видеокарта', Gpu, gpuRows)}{/if}
</div>

<style>
  .tree { overflow: auto; }
  .tree > h3 { display: flex; align-items: center; justify-content: space-between; }
  section { margin-bottom: 6px; }
  .head {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 8px 0;
    border: 0;
    background: none;
    color: #d4dae5;
    font-size: 13px;
  }
  .chev { display: grid; color: var(--dim); transition: transform 0.15s; }
  .chev.closed { transform: rotate(-90deg); }
  ul { list-style: none; margin: 0 0 6px; padding: 0 0 0 4px; }
  li { display: flex; align-items: center; gap: 12px; padding: 2px 6px; border-radius: 8px; }
  li.selected { background: var(--accent-soft); }
  li button {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
    padding: 8px 0;
    border: 0;
    background: none;
    font-size: 13px;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  li.stub button, li.off button { color: var(--dim); }
  i { width: 10px; height: 10px; border-radius: 50%; flex: none; }
  /* карандаш виден при наведении на строку или фокусе с клавиатуры */
  li button.rename { flex: none; display: grid; place-items: center; width: 24px; height: 24px; padding: 0; border-radius: 6px; color: var(--dim); opacity: 0; }
  li:hover button.rename, li button.rename:focus-visible { opacity: 1; }
  li button.rename:hover { color: var(--text); background: rgba(255, 255, 255, 0.08); }
  .name { flex: 1; min-width: 0; margin: 3px 0; padding: 4px 6px; border: 1px solid var(--accent); border-radius: 6px; background: var(--bg); color: var(--text); font: inherit; font-size: 13px; outline: none; }
</style>
