<script lang="ts">
  import { app } from '$lib/state/app.svelte';
  import { apply, reset, toAll, setPower, unsaved, zoneOf, zoneLabel } from '$lib/state/board.svelte';
  import { MODES, MODE_OFF, actual, css } from '$lib/leds';

  const zone = $derived(zoneOf(app.selected));
  const ids = $derived(app.zones.map((z) => z.id));
  const lit = $derived(app.zones.filter((z) => z.mode !== MODE_OFF).length);
  const ready = $derived(app.status === 'ok');
  const changed = $derived(unsaved());
  let saving = $state(false);

  async function save() {
    saving = true;
    await apply();
    saving = false;
  }
</script>

<div class="panel apply">
  <h3>Все зоны</h3>

  <label class="row">
    <span>Подсветка<small>включено {lit} из {app.zones.length}</small></span>
    <!-- Пока включены не все, тумблер выключен и включает все (так советует Adobe Spectrum) -->
    <input
      type="checkbox"
      role="switch"
      class="switch"
      disabled={!ready}
      checked={lit > 0 && lit === app.zones.length}
      onchange={() => setPower(ids, lit !== app.zones.length)}
    />
  </label>

  <div class="copy">
    {#if zone}
      <span class="zone">
        <i style:background={zone.mode === MODE_OFF ? '#2a3140' : css(actual(zone).rgb)}></i>
        <span>{zoneLabel(zone)}<small>{MODES[zone.mode] ?? `Режим ${zone.mode}`}</small></span>
      </span>
    {:else}
      <span class="zone"><small>Выберите зону слева</small></span>
    {/if}
    <button class="btn" disabled={!ready || !zone} onclick={toAll}>На все зоны</button>
  </div>

  <div class="foot">
    <p class="state" class:changed>
      <i></i>{changed ? 'Не сохранено в контроллер' : 'Сохранено в контроллер'}
    </p>
    <div class="buttons">
      <button class="btn" disabled={!ready || !changed} onclick={reset}>Сбросить</button>
      <button class="btn primary" disabled={!ready || !changed || saving} onclick={save}>Сохранить</button>
    </div>
  </div>
</div>

<style>
  .apply { display: flex; flex-direction: column; gap: 12px; min-height: 0; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 12px; cursor: pointer; }
  .row > span, .zone > span { display: grid; gap: 2px; font-size: 13px; }
  small { color: var(--dim); font-size: 12px; }
  .copy { display: flex; align-items: center; gap: 12px; padding: 10px 12px; border: 1px solid var(--border); border-radius: 10px; }
  .zone { display: flex; align-items: center; gap: 10px; flex: 1; min-width: 0; }
  .zone i { width: 12px; height: 12px; border-radius: 50%; flex: none; }
  .copy .btn { padding: 8px 12px; white-space: nowrap; }
  .foot { margin-top: auto; display: grid; gap: 10px; }
  .state { display: flex; align-items: center; gap: 8px; margin: 0; color: var(--dim); font-size: 12px; }
  .state i { width: 8px; height: 8px; border-radius: 50%; background: var(--ok); }
  .state.changed { color: var(--warn); }
  .state.changed i { background: var(--warn); }
  .buttons { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .buttons .btn { padding: 11px; }
</style>
