// Режимы контроллера и приблизительный вид эффектов (порт preview.rs из версии на egui).

export type RGB = [number, number, number];

/** Настройки подсветки зоны так, как их видит интерфейс: цвет хранится на полной яркости. */
export type Look = { mode: number; color: RGB; bright: number; speed: number; palette: RGB[] };

export const MODE_OFF = 0;
export const MODE_STATIC = 1;

export const MODES: Record<number, string> = {
  0: 'Выключено', 1: 'Статичный', 2: 'Дыхание', 3: 'Стробоскоп', 4: 'Смена цветов', 5: 'Случайный',
  7: 'Волна', 8: 'Пружина', 9: 'Стек', 10: 'Cram', 11: 'Сканер', 12: 'Неон', 13: 'Вода',
  14: 'Радуга', 15: 'Радуга 2', 16: 'GlowStream', 17: 'LuminousTide',
};

export const HINTS: Record<number, string> = {
  0: 'Подсветка погашена', 1: 'Постоянный цвет', 2: 'Плавное затухание', 3: 'Ритмичное мигание',
  4: 'Перебор оттенков', 5: 'Случайные вспышки', 7: 'Движение цвета', 8: 'Упругие колебания', 9: 'Набор по светодиоду',
  10: 'Заполнение', 11: 'Бегущая точка', 12: 'Мягкое свечение', 13: 'Переливы', 14: 'Циклическая смена цветов',
  15: 'Радуга, вариант 2', 16: 'Поток по палитре', 17: 'Прилив по палитре',
};

/** Режимы, где контроллер сам перебирает оттенки и не смотрит на цвет */
export const HUE_MODES = [4, 7, 14, 15];
export const PALETTE_MODES = [16, 17];
export const MAX_PALETTE = 11;

export const usesColor = (mode: number) => mode !== MODE_OFF && !HUE_MODES.includes(mode);
export const usesPalette = (mode: number) => PALETTE_MODES.includes(mode);
export const animated = (mode: number) => mode !== MODE_OFF && mode !== MODE_STATIC;

export const PRESETS: RGB[] = [
  [255, 0, 0], [255, 128, 0], [255, 220, 0], [0, 220, 60], [0, 230, 230], [0, 80, 255], [140, 40, 255], [240, 240, 255],
];

const DARK: RGB = [18, 20, 26];

export const scale = (c: RGB, k: number): RGB => c.map((v) => Math.round(v * Math.min(Math.max(k, 0), 1))) as RGB;
export const mix = (a: RGB, b: RGB, k: number): RGB => a.map((v, i) => Math.round(v + (b[i] - v) * k)) as RGB;
export const css = (c: RGB, alpha = 1) => `rgba(${c[0]},${c[1]},${c[2]},${alpha})`;
export const hex = (c: RGB) => '#' + c.map((v) => v.toString(16).padStart(2, '0')).join('');
export const fromHex = (h: string): RGB => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16)) as RGB;

export function hsv(h: number, s: number, v: number): RGB {
  const f = (n: number) => {
    const k = (n + h * 6) % 6;
    return Math.round(255 * (v - v * s * Math.max(0, Math.min(k, 4 - k, 1))));
  };
  return [f(5), f(3), f(1)];
}

export function toHsv([r, g, b]: RGB): [number, number, number] {
  const max = Math.max(r, g, b), d = max - Math.min(r, g, b);
  let h = 0;
  if (d) h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [((h / 6) % 1 + 1) % 1, max ? d / max : 0, max / 255];
}

/** Что реально уходит в контроллер: цвет и палитра, приглушённые яркостью */
export const actual = (l: Look) => ({
  rgb: scale(l.color, l.bright / 100),
  palette: l.palette.map((c) => scale(c, l.bright / 100)),
});

/** Разложить цвет из контроллера на цвет полной яркости и процент яркости */
export function split(rgb: RGB): { color: RGB; bright: number } {
  const [h, s, v] = toHsv(rgb);
  return v ? { color: hsv(h, s, 1), bright: Math.round(v * 100) } : { color: [255, 255, 255], bright: 0 };
}

function gradient(palette: RGB[], x: number): RGB {
  const n = palette.length;
  x = (((x % 1) + 1) % 1) * n;
  const i = Math.floor(x);
  return mix(palette[i % n], palette[(i + 1) % n], x - i);
}

/**
 * Цвет светодиода в точке `pos` (0..1 вдоль ленты или кольца) в момент `t` секунд.
 * shortcut: Пружина/Стек/Cram/Неон/Вода/Случайный показаны пульсацией цвета — их вид на железе не изучен.
 */
export function led(l: Look, pos: number, t: number): RGB {
  const period = 6 + (0.6 - 6) * (l.speed / 100);
  const phase = t / period;
  const { rgb: base, palette } = actual(l);
  const frac = phase - Math.floor(phase);
  switch (l.mode) {
    case 0: return DARK;
    case 1: return base;
    case 2: return scale(base, 0.5 - 0.5 * Math.cos(phase * 2 * Math.PI));
    case 3: return frac < 0.5 ? base : DARK;
    case 4: return hsv(frac, 1, 1);
    case 7: case 14: case 15: return hsv((((pos + phase) % 1) + 1) % 1, 1, 1);
    case 11: return scale(base, 1 - Math.min((((pos - frac) % 1) + 1) % 1 * 4, 1));
    case 16: case 17: return palette.length ? gradient(palette, pos + phase) : base;
    default: return scale(base, 0.75 + 0.25 * Math.sin(phase * 2 * Math.PI));
  }
}
