<script lang="ts">
  import { app, select, placeholders, zoneLabel } from './app.svelte';
  import EffectStrip from './EffectStrip.svelte';
  import { MODES } from './leds';

  const extra = $derived(placeholders());

  function edit(key: string) {
    select(key);
    app.page = 'home';
  }
</script>

<div class="wrap">
  <section class="panel">
    <h3>{app.sys.board ? `${app.sys.board.Manufacturer} ${app.sys.board.Product}` : 'Материнская плата'} · USB-контроллер 26CE:01A2</h3>
    <div class="grid">
      {#each app.zones as z (z.id)}
        <button class="card" onclick={() => edit(`zone:${z.id}`)}>
          <EffectStrip look={z} />
          <b>{zoneLabel(z)}</b>
          <small>Зона {z.id} · {MODES[z.mode] ?? `режим ${z.mode}`}{z.mode > 1 ? ` · ${z.speed}%` : ''}</small>
        </button>
      {/each}
    </div>
  </section>

  <section class="panel">
    <h3>Другие устройства <small>управление подсветкой пока не поддерживается</small></h3>
    <div class="grid">
      {#each app.sys.cpu ?? [] as c (c.Name)}
        <div class="card stub"><b>Процессор</b><small>{c.Name}</small></div>
      {/each}
      {#each [...extra.gpu, ...extra.ram] as d (d.key)}
        <div class="card stub">
          <b>{d.name || 'Устройство'}</b>
          {#each d.info as [k, v] (k)}<small>{k}: {v}</small>{/each}
        </div>
      {/each}
    </div>
  </section>
</div>

<style>
  .wrap { display: grid; gap: 16px; align-content: start; overflow: auto; min-height: 0; }
  h3 small { margin-left: 8px; color: var(--dim); font-weight: 400; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 12px; }
  .card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--card);
    text-align: left;
  }
  button.card:hover { background: var(--card-hover); }
  .card b { margin-top: 6px; font-weight: 600; }
  .card small { color: var(--dim); font-size: 12px; }
</style>
