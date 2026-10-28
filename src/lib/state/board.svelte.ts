// Плата ASRock (src-tauri/src/devices/asrock): зоны, правка выбранной зоны, запись в контроллер.

import { api } from '$lib/api';
import { actual, split, MODE_OFF, MODE_STATIC, type Look, type RGB } from '$lib/leds';
import { app, fail, type Zone } from './app.svelte';
import { followBoard } from './keyboard.svelte';

/** На этой плате кулеры висят на ARGB #1 (проверено глазами) */
export const FANS_ZONE = 2;
export const defaultLabel = (z: Zone) => (z.id === FANS_ZONE ? `Вентиляторы (${z.name})` : z.name);
export const zoneLabel = (z: Zone) => app.names[z.id] || defaultLabel(z);

export const look = (l: Look): Look => ({ mode: l.mode, color: [...l.color], bright: l.bright, speed: l.speed, palette: l.palette.map((c) => [...c] as RGB) });
/** Одинаково ли выглядят на железе: сравниваем то, что реально уходит в контроллер */
export const same = (a: Look, b: Look) => JSON.stringify([a.mode, a.speed, actual(a)]) === JSON.stringify([b.mode, b.speed, actual(b)]);

export async function connect() {
  app.status = 'connecting';
  try {
    const zones = await api.connect();
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

export const zoneOf = (key: string) => app.zones.find((z) => `zone:${z.id}` === key);

export function select(key: string) {
  app.selected = key;
  const zone = zoneOf(key);
  if (zone) app.edit = look(zone);
}

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
      const { rgb, palette } = actual(z);
      await api.writeZone(id, z.mode, z.speed, rgb, palette);
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

/** Ctrl+Alt+L: погасить всё, а если всё погашено — включить */
export function togglePower() {
  if (app.status !== 'ok') return;
  setPower(app.zones.map((z) => z.id), app.zones.every((z) => z.mode === MODE_OFF));
}

/** «Сохранить»: всё, что уже видно на железе, запомнить в контроллере */
export async function apply() {
  while (busy) await new Promise((r) => setTimeout(r, 20));
  try {
    await api.save();
    app.saved = app.zones.map((z) => ({ ...z, ...look(z) }));
  } catch (e) {
    fail(e);
  }
}

export const reset = () => setLooks((zone) => app.saved.find((s) => s.id === zone.id));
