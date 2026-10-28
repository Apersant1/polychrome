// Живые эффекты «Музыка» и «Экран» (src-tauri/src/live.rs) поверх всех устройств.

import { api, type Live } from '$lib/api';
import { app, fail } from './app.svelte';

/** Повторный выбор того же эффекта выключает его */
export async function toggleLive(src: Live) {
  const next = app.live === src ? null : src;
  try {
    await api.setLive(next);
    app.live = next;
  } catch (e) {
    fail(e);
  }
}
