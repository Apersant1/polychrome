<script lang="ts">
  import { app, connect } from './app.svelte';
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
    <p>Если закрыть окно с незаписанными изменениями, программа запишет их сама. «Сбросить» возвращает то, что сейчас лежит в контроллере.</p>
    <p>Профили хранятся на этом компьютере.</p>
  </section>
</div>

<style>
  .wrap { display: grid; gap: 16px; align-content: start; max-width: 760px; }
  dl { display: grid; grid-template-columns: 160px 1fr; gap: 10px; margin: 0 0 18px; }
  dt { color: var(--dim); }
  dd { margin: 0; }
  p { margin: 0 0 10px; color: #c4ccda; line-height: 1.5; }
</style>
