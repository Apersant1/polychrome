// config.json рядом с exe (src-tauri/src/config.rs): профили, имена зон, настройки клавиатуры. Его же читает --profile.

import { api } from '$lib/api';
import type { Look } from '$lib/leds';
import { app, KBD_STATIC, type Profile } from './app.svelte';

type Config = { profiles: Profile[]; names?: Record<string, string>; kbdFollow?: boolean; kbdMode?: number };

const OLD_KEY = 'polychrome.profiles'; // раньше профили лежали в localStorage окна
const preset = (name: string, l: Look): Profile => ({ name, zones: { '*': l } });
const DEFAULT_PROFILES: Profile[] = [
  preset('Геймерский', { mode: 14, color: [255, 0, 0], bright: 100, speed: 70, palette: [] }),
  preset('Рабочий', { mode: 1, color: [200, 225, 255], bright: 60, speed: 50, palette: [] }),
  preset('Вечерний', { mode: 1, color: [255, 110, 20], bright: 40, speed: 50, palette: [] }),
  preset('Дыхание', { mode: 2, color: [140, 40, 255], bright: 100, speed: 35, palette: [] }),
];

let persist = true; // файл не прочитался — не затираем его тем, что в памяти

export async function loadConfig() {
  try {
    const cfg = await api.loadConfig<Config>();
    if (cfg) {
      app.profiles = cfg.profiles;
      app.names = cfg.names ?? {};
      app.kbdFollow = cfg.kbdFollow ?? false;
      app.kbdMode = cfg.kbdMode ?? KBD_STATIC;
      return;
    }
    try {
      app.profiles = JSON.parse(localStorage.getItem(OLD_KEY) ?? 'null') ?? DEFAULT_PROFILES;
    } catch {
      app.profiles = DEFAULT_PROFILES;
    }
    storeConfig(); // сразу в файл, чтобы --profile видел профили
  } catch (e) {
    persist = false;
    app.profiles = DEFAULT_PROFILES;
    alert(`${e}\nПрофили по умолчанию, изменения не сохранятся.`);
  }
}

export function storeConfig() {
  if (!persist) return;
  const config: Config = { profiles: app.profiles, names: app.names, kbdFollow: app.kbdFollow, kbdMode: app.kbdMode };
  api.saveConfig($state.snapshot(config)).catch((e) => alert(e));
}

/** Пустое имя — вернуть стандартное */
export function renameZone(id: number, name: string) {
  if (name.trim()) app.names[id] = name.trim();
  else delete app.names[id];
  storeConfig();
}
