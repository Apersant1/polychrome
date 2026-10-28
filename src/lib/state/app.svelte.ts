// Общее состояние интерфейса. Логика — в соседних модулях по предметным областям:
// board (плата), keyboard (клавиатура ZET), profiles, config (config.json), live, system.
// Зависимости идут в одну сторону: config ← keyboard ← board ← profiles.

import type { KbdEffect, KbdView, Live, SysInfo } from '$lib/api';
import { MODE_STATIC, type Look } from '$lib/leds';

export type Zone = Look & { id: number; name: string };
export type Profile = { name: string; zones: Record<string, Look>; kbd?: KbdEffect }; // ключ — номер зоны или '*' для всех
export type Page = 'home' | 'devices' | 'effects' | 'profiles' | 'settings';
/** Чем светится деталь на картинке компьютера: номер зоны, null — без подсветки, 'hidden' — такой детали нет */
export type PartZone = number | null | 'hidden';

export const KBD_STATIC = 1;

export const app = $state({
  page: 'home' as Page,
  status: 'connecting' as 'connecting' | 'ok' | 'error',
  error: '',
  zones: [] as Zone[],
  saved: [] as Zone[], // что лежит в памяти контроллера — для «Сбросить»
  selected: '' as string, // 'zone:2', 'kbd', 'gpu:0', 'ram:1'
  edit: { mode: MODE_STATIC, color: [0, 120, 255], bright: 100, speed: 50, palette: [] } as Look,
  sys: {} as SysInfo,
  profiles: [] as Profile[],
  profile: '',
  names: {} as Record<string, string>, // свои имена зон по номеру
  scene: {} as Record<string, PartZone>, // детали картинки компьютера, которые пользователь перенастроил (Preview.svelte)
  fanCounts: {} as Record<string, number>, // сколько вентиляторов в группе на картинке, если не 3
  live: null as Live | null, // живой эффект поверх всех устройств (live.rs)
  kbd: null as KbdView | null, // null — клавиатуры нет или её держит Blade Pro Software
  kbdFollow: false, // статичный цвет клавиатуры повторяет правки зон
  kbdMode: KBD_STATIC, // как прочитать включённый режим клавиатуры, неизвестно — помним последний выбранный здесь
  now: 0, // секунды, общий такт анимаций
});

/** Ошибка связи с платой: показывается баннером с кнопкой «Повторить» */
export function fail(e: unknown) {
  app.status = 'error';
  app.error = String(e);
}
