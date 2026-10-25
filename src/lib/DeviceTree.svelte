<script lang="ts">
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import CircuitBoard from '@lucide/svelte/icons/circuit-board';
  import Fan from '@lucide/svelte/icons/fan';
  import MemoryStick from '@lucide/svelte/icons/memory-stick';
  import Gpu from '@lucide/svelte/icons/gpu';
  import { app, select, placeholders, setPower, zoneLabel, FANS_ZONE, type Zone } from './app.svelte';
  import { css, actual, MODE_OFF } from './leds';

  const extra = $derived(placeholders());
  const board = $derived(app.zones.filter((z) => z.id !== FANS_ZONE));
  const fans = $derived(app.zones.filter((z) => z.id === FANS_ZONE));
  let closed = $state<Record<string, boolean>>({});

  const dot = (z: Zone) => (z.mode === MODE_OFF ? '#2a3140' : css(actual(z).rgb));
</script>

{#snippet zoneRow(z: Zone)}
  <li class:selected={app.selected === `zone:${z.id}`} class:off={z.mode === MODE_OFF}>
    <button onclick={() => select(`zone:${z.id}`)}>
      <i style:background={dot(z)} style:box-shadow="0 0 8px {dot(z)}"></i>{zoneLabel(z)}
    </button>
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

{#snippet group(id: string, title: string, Icon: typeof Fan, body: import('svelte').Snippet)}
  <section>
    <button class="head" onclick={() => (closed[id] = !closed[id])}>
      <span class="chev" class:closed={closed[id]}><ChevronDown size={14} /></span>
      <Icon size={17} />{title}
    </button>
    {#if !closed[id]}<ul>{@render body()}</ul>{/if}
  </section>
{/snippet}

{#snippet boardRows()}
  {#each board as z (z.id)}{@render zoneRow(z)}{/each}
{/snippet}
{#snippet fanRows()}
  {#each fans as z (z.id)}{@render zoneRow(z)}{/each}
{/snippet}
{#snippet ramRows()}
  {#each extra.ram as d (d.key)}
    <li class="stub" class:selected={app.selected === d.key}>
      <button onclick={() => select(d.key)}>{d.name || 'Модуль памяти'}</button>
    </li>
  {/each}
{/snippet}
{#snippet gpuRows()}
  {#each extra.gpu as d (d.key)}
    <li class="stub" class:selected={app.selected === d.key}>
      <button onclick={() => select(d.key)}>{d.name}</button>
    </li>
  {/each}
{/snippet}

<div class="panel tree">
  <h3>Устройства</h3>
  {@render group('board', app.sys.board?.Product ?? 'Материнская плата', CircuitBoard, boardRows)}
  {#if fans.length}{@render group('fans', 'Охлаждение', Fan, fanRows)}{/if}
  {#if extra.ram.length}{@render group('ram', 'Память', MemoryStick, ramRows)}{/if}
  {#if extra.gpu.length}{@render group('gpu', 'Видеокарта', Gpu, gpuRows)}{/if}
</div>

<style>
  .tree { overflow: auto; }
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
</style>
