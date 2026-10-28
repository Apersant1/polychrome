import { invoke } from '@tauri-apps/api/core';
import { actual, split, MODE_OFF, MODE_STATIC, type Look, type RGB } from './leds';

export type Zone = Look & { id: number; name: string };
export type Profile = { name: string; zones: Record<string, Look>; kbd?: KbdEffect }; // ключ — номер зоны или '*' для всех
/** Клавиатура ZET (zet.rs): режим 1 статичный … 13 своя раскраска; цвет 0xRRGGBB, бит 24 — радуга */
export type KbdEffect = { mode: number; brightness: number; speed: number; direction: number; palette: number[] };
export type KbdMode = { index: number; speeds: number; brights: number; dirs: number; colors: number };
export type Live = 'music' | 'screen';
export type Page = 'home' | 'devices' | 'effects' | 'profiles' | 'settings';
export type SysInfo = {
  board?: { Manufacturer: string; Product: string };
  cpu?: { Name: string }[];
  gpu?: { Name: string; DriverVersion: string }[];
  ram?: { Manufacturer: string; PartNumber: string; Capacity: number; ConfiguredClockSpeed: number; DeviceLocator: string }[];
};

/** На этой плате кулеры висят на ARGB #1 (проверено глазами) */
export const FANS_ZONE = 2;
export const defaultLabel = (z: Zone) => (z.id === FANS_ZONE ? `Вентиляторы (${z.name})` : z.name);
export const zoneLabel = (z: Zone) => app.names[z.id] || defaultLabel(z);

const look = (l: Look): Look => ({ mode: l.mode, color: [...l.color], bright: l.bright, speed: l.speed, palette: l.palette.map((c) => [...c] as RGB) });
/** Одинаково ли выглядят на железе: сравниваем то, что реально уходит в контроллер */
const same = (a: Look, b: Look) => JSON.stringify([a.mode, a.speed, actual(a)]) === JSON.stringify([b.mode, b.speed, actual(b)]);

export const app = $state({
  page: 'home' as Page,
  status: 'connecting' as 'connecting' | 'ok' | 'error',
  error: '',
  zones: [] as Zone[],
  saved: [] as Zone[], // что лежит в памяти контроллера — для «Сбросить»
  selected: '' as string, // 'zone:2', 'gpu:0', 'ram:1'
  edit: { mode: MODE_STATIC, color: [0, 120, 255], bright: 100, speed: 50, palette: [] } as Look,
  sys: {} as SysInfo,
  profiles: [] as Profile[],
  names: {} as Record<string, string>, // свои имена зон по номеру
  live: null as Live | null, // живой эффект поверх всех зон (live.rs)
  kbd: null as { modes: KbdMode[]; effect: KbdEffect } | null, // null — нет службы Blade Pro Software
  kbdFollow: false, // статичный цвет клавиатуры повторяет правки зон
  profile: '',
  now: 0, // секунды, общий такт анимаций
});

export async function connect() {
  app.status = 'connecting';
  try {
    const zones = await invoke<{ id: number; name: string; mode: number; rgb: RGB; speed: number; palette: RGB[] }[]>('connect');
    app.zones = zones.map((z) => ({ id: z.id, name: z.name, mode: z.mode, speed: z.speed, palette: z.palette, ...split(z.rgb) }));
    app.saved = app.zones.map((z) => ({ ...z, ...look(z) }));
    const first = app.zones.find((z) => z.id === FANS_ZONE) ?? app.zones[0];
    if (first) select(`zone:${first.id}`);
    app.status = 'ok';
    app.error = '';
  } catch (e) {
    fail(e);
  }
}

/** Есть ли на железе то, чего нет в памяти контроллера */
export const unsaved = () => app.zones.some((z) => { const s = app.saved.find((s) => s.id === z.id); return !s || !same(z, s); });

function fail(e: unknown) {
  app.status = 'error';
  app.error = String(e);
}

export function select(key: string) {
  app.selected = key;
  const zone = zoneOf(key);
  if (zone) app.edit = look(zone);
}

export const zoneOf = (key: string) => app.zones.find((z) => `zone:${z.id}` === key);


// Запись в контроллер по одной команде: при перетаскивании цвета остаётся только последнее значение зоны
const pending = new Map<number, Zone>();
let busy = false;

function push(zone: Zone) {
  pending.set(zone.id, $state.snapshot(zone) as Zone);
  if (!busy) flush();
}

async function flush() {
  busy = true;
  while (pending.size) {
    const [id, z] = pending.entries().next().value!;
    pending.delete(id);
    try {
      await invoke('write_zone', { id, mode: z.mode, speed: z.speed, ...actual(z) });
    } catch (e) {
      pending.clear();
      fail(e);
    }
  }
  busy = false;
}

/** Поменять настройки выбранной зоны и сразу показать их на железе */
export function change(patch: Partial<Look>) {
  Object.assign(app.edit, patch);
  const zone = zoneOf(app.selected);
  if (!zone) return;
  Object.assign(zone, look(app.edit));
  push(zone);
  if (app.kbdFollow) followBoard(app.edit);
}

export function setLooks(byZone: (zone: Zone) => Look | undefined) {
  for (const zone of app.zones) {
    const l = byZone(zone);
    if (!l) continue;
    Object.assign(zone, look(l));
    push(zone);
  }
  select(app.selected);
}

/** Настройки выбранной зоны — на все зоны */
export const toAll = () => setLooks(() => app.edit);

const lastOn = new Map<number, Look>(); // чем зона светилась до выключения
const DEFAULT_ON: Look = { mode: MODE_STATIC, color: [255, 255, 255], bright: 100, speed: 50, palette: [] };

export function setPower(ids: number[], on: boolean) {
  setLooks((z) => {
    if (!ids.includes(z.id) || (z.mode !== MODE_OFF) === on) return;
    if (on) return lastOn.get(z.id) ?? DEFAULT_ON;
    lastOn.set(z.id, look(z));
    return { ...look(z), mode: MODE_OFF };
  });
}

/** «Сохранить»: всё, что уже видно на железе, запомнить в контроллере */
export async function apply() {
  while (busy) await new Promise((r) => setTimeout(r, 20));
  try {
    await invoke('save');
    app.saved = app.zones.map((z) => ({ ...z, ...look(z) }));
  } catch (e) {
    fail(e);
  }
}

export const reset = () => setLooks((zone) => app.saved.find((s) => s.id === zone.id));

// Профили и имена зон живут в config.json (см. config_path в lib.rs), его же читает --profile
const OLD_KEY = 'polychrome.profiles'; // раньше профили лежали в localStorage окна
const preset = (name: string, l: Look): Profile => ({ name, zones: { '*': l } });
const SEED: Profile[] = [
  preset('Геймерский', { mode: 14, color: [255, 0, 0], bright: 100, speed: 70, palette: [] }),
  preset('Рабочий', { mode: 1, color: [200, 225, 255], bright: 60, speed: 50, palette: [] }),
  preset('Вечерний', { mode: 1, color: [255, 110, 20], bright: 40, speed: 50, palette: [] }),
  preset('Дыхание', { mode: 2, color: [140, 40, 255], bright: 100, speed: 35, palette: [] }),
];

let persist = true; // файл не прочитался — не затираем его тем, что в памяти

export async function loadConfig() {
  try {
    const cfg = await invoke<{ profiles: Profile[]; names?: Record<string, string>; kbdFollow?: boolean; kbdMode?: number } | null>('load_config');
    if (cfg) {
      app.profiles = cfg.profiles;
      app.names = cfg.names ?? {};
      app.kbdFollow = cfg.kbdFollow ?? false;
      kbdMode = cfg.kbdMode ?? KBD_STATIC;
      return;
    }
    try {
      app.profiles = JSON.parse(localStorage.getItem(OLD_KEY) ?? 'null') ?? SEED;
    } catch {
      app.profiles = SEED;
    }
    storeConfig(); // сразу в файл, чтобы --profile видел профили
  } catch (e) {
    persist = false;
    app.profiles = SEED;
    alert(`${e}
Профили по умолчанию, изменения не сохранятся.`);
  }
}

function storeConfig() {
  if (!persist) return;
  const config = { profiles: app.profiles, names: app.names, kbdFollow: app.kbdFollow, kbdMode };
  invoke('save_config', { config: $state.snapshot(config) }).catch((e) => alert(e));
}

/** Пустое имя — вернуть стандартное */
export function renameZone(id: number, name: string) {
  if (name.trim()) app.names[id] = name.trim();
  else delete app.names[id];
  storeConfig();
}

export const profileLook = (p: Profile, id: number) => p.zones[id] ?? p.zones['*'];

/** Выбор профиля — осознанное действие, поэтому сразу пишем его в контроллер */
export function useProfile(name: string) {
  const p = app.profiles.find((p) => p.name === name);
  if (!p) return;
  app.profile = name;
  setLooks((zone) => profileLook(p, zone.id));
  if (p.kbd) kbdSet(p.kbd);
  return apply();
}

/** Зоны или клавиатура ушли от того, что записано в профиле */
export const profileChanged = (p: Profile) =>
  app.zones.some((z) => { const l = profileLook(p, z.id); return l && !same(z, l); }) ||
  (!!p.kbd && !!app.kbd && JSON.stringify(p.kbd) !== JSON.stringify(app.kbd.effect));

export const profileExists = (name: string) => app.profiles.some((p) => p.name === name);

/** Записать текущие зоны в профиль (новый или существующий с тем же именем) */
export function saveProfile(name: string) {
  const zones = Object.fromEntries(app.zones.map((z) => [z.id, look(z)]));
  const i = app.profiles.findIndex((p) => p.name === name);
  const kbd = app.kbd ? ($state.snapshot(app.kbd.effect) as KbdEffect) : app.profiles[i]?.kbd;
  if (i < 0) app.profiles.push({ name, zones, kbd });
  else Object.assign(app.profiles[i], { zones, kbd });
  app.profile = name;
  storeConfig();
}

export function renameProfile(from: string, to: string) {
  const p = app.profiles.find((p) => p.name === from);
  if (!p || profileExists(to)) return;
  p.name = to;
  if (app.profile === from) app.profile = to;
  storeConfig();
}

/** Удаляет и возвращает функцию «Вернуть» */
export function deleteProfile(name: string) {
  const i = app.profiles.findIndex((p) => p.name === name);
  if (i < 0) return;
  const [p] = app.profiles.splice(i, 1);
  const wasActive = app.profile === name;
  if (wasActive) app.profile = '';
  storeConfig();
  return () => {
    if (profileExists(p.name)) return;
    app.profiles.splice(Math.min(i, app.profiles.length), 0, p);
    if (wasActive) app.profile = p.name;
    storeConfig();
  };
}

/** Повторный выбор того же эффекта выключает его */
export async function toggleLive(src: Live) {
  const next = app.live === src ? null : src;
  try {
    await invoke('set_live', { source: next });
    app.live = next;
  } catch (e) {
    fail(e);
  }
}

const KBD_STATIC = 1;
let kbdMode = KBD_STATIC; // служба не сообщает включённый режим — помним последний выбранный здесь

/** Клавиатура ZET — только если стоит Blade Pro Software со своей службой */
export async function openKbd() {
  try {
    app.kbd = await invoke('kbd_open', { mode: kbdMode });
  } catch {
    app.kbd = null;
  }
}

// Как push() для зон: при перетаскивании цвета уходит только последнее значение
let kbdPending: KbdEffect | null = null;
let kbdBusy = false;

export async function kbdSet(effect: KbdEffect) {
  if (!app.kbd) return;
  app.kbd.effect = effect;
  if (effect.mode !== kbdMode) {
    kbdMode = effect.mode;
    storeConfig();
  }
  kbdPending = $state.snapshot(effect) as KbdEffect;
  if (kbdBusy) return;
  kbdBusy = true;
  while (kbdPending) {
    const e = kbdPending;
    kbdPending = null;
    try {
      await invoke('kbd_set', { effect: e });
    } catch {
      app.kbd = null; // служба пропала — панель клавиатуры скроется
      kbdPending = null;
    }
  }
  kbdBusy = false;
}

/** Сменить режим: ползунки и цвета берём те, что клавиатура помнит для этого режима */
export async function kbdSetMode(mode: number) {
  try {
    await kbdSet(await invoke<KbdEffect>('kbd_effect', { mode }));
  } catch {
    app.kbd = null;
  }
}

export function setKbdFollow(on: boolean) {
  app.kbdFollow = on;
  storeConfig();
  if (on) followBoard(app.edit);
}

export const rgbToInt = ([r, g, b]: RGB) => (r << 16) | (g << 8) | b;

/** «Цвет как у платы»: статичный режим цветом зоны; выключенная зона — чёрным */
function followBoard(l: Look) {
  const m = app.kbd?.modes.find((m) => m.index === KBD_STATIC);
  if (!m) return;
  const color = l.mode === MODE_OFF ? 0 : rgbToInt(actual(l).rgb);
  kbdSet({ mode: KBD_STATIC, brightness: m.brights - 1, speed: 0, direction: 0, palette: [color] });
}

/** Ctrl+Alt+L: погасить всё, а если всё погашено — включить */
export function togglePower() {
  if (app.status !== 'ok') return;
  setPower(app.zones.map((z) => z.id), app.zones.every((z) => z.mode === MODE_OFF));
}

/** Ctrl+Alt+P: следующий профиль по кругу */
export function nextProfile() {
  if (app.status !== 'ok' || !app.profiles.length) return;
  const i = app.profiles.findIndex((p) => p.name === app.profile);
  return useProfile(app.profiles[(i + 1) % app.profiles.length].name);
}

export async function loadSystem() {
  try {
    app.sys = await invoke<SysInfo>('system_info');
  } catch {
    app.sys = {}; // не критично: заглушки просто останутся без моделей
  }
}

/** Устройства, которыми пока не управляем: только показываем, что это */
export function placeholders() {
  const s = app.sys;
  return {
    gpu: (s.gpu ?? []).map((g, i) => ({ key: `gpu:${i}`, name: g.Name, info: [['Драйвер', g.DriverVersion]] })),
    ram: (s.ram ?? []).map((m, i) => ({
      key: `ram:${i}`,
      name: `${m.Manufacturer} ${m.PartNumber}`.trim(),
      info: [['Слот', m.DeviceLocator], ['Объём', `${Math.round(m.Capacity / 2 ** 30)} ГБ`], ['Частота', `${m.ConfiguredClockSpeed} МГц`]],
    })),
  };
}
