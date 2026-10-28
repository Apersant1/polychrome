<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { app } from '$lib/state/app.svelte';
  import { connect, defaultLabel } from '$lib/state/board.svelte';
  import { renameZone } from '$lib/state/config.svelte';

  let autostart = $state(false);
  onMount(() => api.getAutostart().then((on) => (autostart = on)));

  function setAutostart(on: boolean) {
    api.setAutostart(on).then(() => (autostart = on), (e) => alert(e));
  }
</script>

<div class="wrap">
  <section class="panel">
    <h3>Контроллер</h3>
    <dl>
      <dt>Устройство</dt><dd>ASRock Polychrome USB, 26CE:01A2</dd>
      <dt>Состояние</dt><dd>{app.status === 'ok' ? 'подключён' : app.status === 'connecting' ? 'подключение…' : app.error}</dd>
      <dt>Активные зоны</dt><dd>{app.zones.map((z) => z.name).join(', ') || '—'}</dd>
    </dl>
    <button class="btn" onclick={connect}>Переподключить</button>
  </section>
  <section class="panel">
    <h3>Как сохраняются настройки</h3>
    <p>Изменения сразу видны на подсветке. «Применить» записывает их в память контроллера, и они переживают перезагрузку без этой программы.</p>
    <p>«Сбросить» возвращает то, что сейчас лежит в контроллере. При выходе из программы незаписанные изменения записываются сами.</p>
  </section>
  <section class="panel">
    <h3>Работа в фоне</h3>
    <label class="row">
      <input type="checkbox" role="switch" class="switch" checked={autostart} onchange={(e) => setAutostart(e.currentTarget.checked)} />
      Запускать вместе с Windows (сразу в трей)
    </label>
    <p>Крестик сворачивает окно в трей, выход — через меню значка в трее.</p>
    <p>Пока программа запущена, подсветка гаснет при блокировке компьютера, во сне и при выключенном мониторе, а потом загорается снова. В память контроллера погашенный вид не записывается.</p>
  </section>
  <section class="panel">
    <h3>Имена зон</h3>
    <dl>
      {#each app.zones as z (z.id)}
        <dt><label for="zone-name-{z.id}">Зона {z.id} · {z.name}</label></dt>
        <dd><input id="zone-name-{z.id}" placeholder={defaultLabel(z)} value={app.names[z.id] ?? ''} onchange={(e) => renameZone(z.id, e.currentTarget.value)} /></dd>
      {/each}
    </dl>
    <p>Пустое поле — стандартное имя.</p>
  </section>
  <section class="panel">
    <h3>Горячие клавиши</h3>
    <dl>
      <dt><kbd>Ctrl+Alt+L</kbd></dt><dd>Погасить всю подсветку или включить обратно</dd>
      <dt><kbd>Ctrl+Alt+P</kbd></dt><dd>Следующий профиль</dd>
    </dl>
    <p>Работают из любой программы, пока Polychrome запущен (в том числе свёрнутым в трей).</p>
  </section>
  <section class="panel">
    <h3>Профили и расписание</h3>
    <p>Профили и имена зон лежат в файле <code>config.json</code> рядом с программой: переносите их вместе.</p>
    <p>Профиль включается и без окна: <code>Polychrome.exe --profile "Вечерний"</code>. Это можно поставить в Планировщик заданий Windows, например на 22:00.</p>
    <button class="btn" onclick={() => api.openConfigDir().catch((e) => alert(e))}>Открыть папку программы</button>
  </section>
</div>

<style>
  /* max-content: у .panel min-height: 0, и без этого строки сжимаются под окно вместо прокрутки */
  .wrap { display: grid; grid-auto-rows: max-content; gap: 16px; align-content: start; max-width: 760px; overflow: auto; min-height: 0; }
  dl { display: grid; grid-template-columns: 160px 1fr; gap: 10px; margin: 0 0 18px; }
  dt { color: var(--dim); }
  dd { margin: 0; }
  p { margin: 0 0 10px; color: #c4ccda; line-height: 1.5; }
  .row { display: flex; align-items: center; gap: 12px; margin-bottom: 14px; }
  input:not(.switch) { width: 100%; max-width: 320px; padding: 7px 10px; border: 1px solid var(--border-strong); border-radius: 8px; background: var(--card); color: var(--text); font: inherit; outline: none; }
  input:not(.switch):focus { border-color: var(--accent); }
  dl { align-items: center; }
  code, kbd { font-family: ui-monospace, monospace; font-size: 12px; color: var(--text); }
</style>
