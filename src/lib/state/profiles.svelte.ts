// Профили: наборы настроек всех зон и клавиатуры, хранятся в config.json.

import type { KbdEffect } from '$lib/api';
import { actual, css, hsv, usesColor, usesPalette, MODE_OFF } from '$lib/leds';
import { app, type Profile } from './app.svelte';
import { apply, look, same, setLooks } from './board.svelte';
import { storeConfig } from './config.svelte';
import { kbdSet, setKbdFollow } from './keyboard.svelte';

export const profileLook = (p: Profile, id: number) => p.zones[id] ?? p.zones['*'];

export const profileExists = (name: string) => app.profiles.some((p) => p.name === name);

/** Полоска цветов профиля по зонам (CSS-градиент): статичная подсказка, а не анимация */
export function profileSwatch(p: Profile) {
  const stops = app.zones.flatMap((z) => {
    const l = profileLook(p, z.id);
    if (!l || l.mode === MODE_OFF) return ['#1a212c'];
    if (usesPalette(l.mode)) return actual(l).palette.map((c) => css(c));
    if (!usesColor(l.mode)) return [0, 0.33, 0.66].map((h) => css(hsv(h, 1, 1)));
    return [css(actual(l).rgb)];
  });
  return `linear-gradient(90deg, ${(stops.length > 1 ? stops : [...stops, ...stops]).join(', ')})`;
}

/** Выбор профиля — осознанное действие, поэтому сразу пишем его в контроллер */
export function useProfile(name: string) {
  const p = app.profiles.find((p) => p.name === name);
  if (!p) return;
  app.profile = name;
  setLooks((zone) => profileLook(p, zone.id));
  if (p.kbd) {
    if (app.kbdFollow) setKbdFollow(false); // своя клавиатура профиля — уже не «Как у платы»
    kbdSet(p.kbd);
  }
  return apply();
}

/** Ctrl+Alt+P: следующий профиль по кругу */
export function nextProfile() {
  if (app.status !== 'ok' || !app.profiles.length) return;
  const i = app.profiles.findIndex((p) => p.name === app.profile);
  return useProfile(app.profiles[(i + 1) % app.profiles.length].name);
}

/** Зоны или клавиатура ушли от того, что записано в профиле */
export const profileChanged = (p: Profile) =>
  app.zones.some((z) => { const l = profileLook(p, z.id); return l && !same(z, l); }) ||
  (!!p.kbd && !!app.kbd && JSON.stringify(p.kbd) !== JSON.stringify(app.kbd.effect));

/** Записать текущие зоны и клавиатуру в профиль (новый или существующий с тем же именем) */
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
