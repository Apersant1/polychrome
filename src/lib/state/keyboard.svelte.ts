// Клавиатура ZET Blade Pro (src-tauri/src/devices/zet).

import { api, type KbdEffect } from '$lib/api';
import { actual, MODE_OFF, type Look, type RGB } from '$lib/leds';
import { app, KBD_STATIC } from './app.svelte';
import { storeConfig } from './config.svelte';

// Номера режимов клавиатуры → KB_MODE_* из прошивки Whiftec (kb_def.py)
export const KBD_NAMES = ['', 'Статичный', 'Дыхание', 'Поток', 'Дождь', 'Бегущий огонь', 'Звёзды', 'Реакция на нажатие', 'Капли', 'Крест', 'Рябь', 'Аврора', 'Тепловая карта', 'Своя раскраска'];
export const KBD_HINTS = ['', 'Постоянный цвет', 'Плавное затухание', 'Цвет течёт по клавишам', 'Падающие капли', 'Огонь бежит по рядам', 'Мерцание клавиш', 'Вспышка от нажатия', 'Круги по клавишам', 'Лучи от нажатой клавиши', 'Волна от нажатия', 'Плавные переливы', 'Ярче, где чаще жмут', 'Цвета клавиш из памяти'];

/** Открыть, если подключена и не занята Blade Pro Software; иначе панель клавиатуры просто не появится */
export async function openKbd() {
  try {
    app.kbd = await api.kbdOpen(app.kbdMode);
  } catch {
    app.kbd = null;
  }
}

// Запись по одной команде: при перетаскивании цвета уходит только последнее значение
let pending: KbdEffect | null = null;
let busy = false;

export async function kbdSet(effect: KbdEffect) {
  if (!app.kbd) return;
  app.kbd.effect = effect;
  if (effect.mode !== app.kbdMode) {
    app.kbdMode = effect.mode;
    storeConfig();
  }
  pending = $state.snapshot(effect) as KbdEffect;
  if (busy) return;
  busy = true;
  while (pending) {
    const e = pending;
    pending = null;
    try {
      await api.kbdSet(e);
    } catch {
      app.kbd = null; // клавиатура пропала — панель скроется
      pending = null;
    }
  }
  busy = false;
}

/** Сменить режим: ползунки и цвета берём те, что клавиатура помнит для этого режима */
export async function kbdSetMode(mode: number) {
  if (app.kbdFollow) setKbdFollow(false);
  try {
    await kbdSet(await api.kbdEffect(mode));
  } catch {
    app.kbd = null;
  }
}

export function setKbdFollow(on: boolean) {
  app.kbdFollow = on;
  storeConfig();
  if (on) followBoard(app.edit);
}

const rgbToInt = ([r, g, b]: RGB) => (r << 16) | (g << 8) | b;

/** «Цвет как у платы»: статичный режим цветом зоны; выключенная зона — чёрным */
export function followBoard(l: Look) {
  const m = app.kbd?.modes.find((m) => m.index === KBD_STATIC);
  if (!m) return;
  const color = l.mode === MODE_OFF ? 0 : rgbToInt(actual(l).rgb);
  kbdSet({ mode: KBD_STATIC, brightness: m.brights - 1, speed: 0, direction: 0, palette: [color] });
}
