// Клавиатура ZET Blade Pro (K191DNS): что знаем о режимах из описания прошивки
// (res/wkb_device_cfg.zip → WKB-K191DNS.cfg в Blade Pro Software) и превью режимов по нему. Анимация приблизительная.

import { css, hsv, mix, scale, type RGB } from './leds';

// Флаги цвета, старший байт: прошивка сама перебирает оттенки / берёт случайный цвет на каждое срабатывание
export const HUE = 0x01000000;
export const RANDOM = 0x14000000; // KB_F_RANDOM | KB_F_REPICK_FG
const FLAGS = 0xff000000;

/** Какой флаг можно включить каждому цвету режима (как в Blade Pro Software); у двухцветных первый цвет — фон */
export const COLOR_FLAGS: Record<number, number[]> = {
  1: [HUE],
  2: [0, RANDOM],
  3: [0, HUE],
  4: [0, HUE],
  5: [0, HUE],
  6: [0, HUE],
  7: [0, RANDOM],
  8: [0, RANDOM],
  9: [0, RANDOM],
  10: [0, RANDOM],
  11: [0, RANDOM],
  12: [0, 0],
  13: [0],
};

/** Ступени яркости в процентах; у своей раскраски восьмая — «динамическая» */
export const BRIGHTS = [0, 16, 32, 48, 65, 84, 100];
/** Ступени скорости в процентах от обычной */
const SPEEDS = [50, 70, 85, 100, 150, 200, 250];
// Как в прошивке: случайный цвет — из этой таблицы
const RANDOM_TAB: RGB[] = [[255, 0, 0], [255, 165, 0], [255, 255, 0], [128, 255, 0], [0, 255, 0], [0, 255, 255], [0, 0, 255], [165, 0, 255], [255, 0, 255], [255, 255, 255]];

export const rgb = (c: number): RGB => [(c >> 16) & 255, (c >> 8) & 255, c & 255];

/** Палитра для карточки режима, который сейчас не выбран: чёрный фон и красный с доступным флагом — видно, что умеет режим */
export function demoPalette(mode: number): number[] {
  if (mode === 12) return [0x0000ff, 0xff0000];
  return (COLOR_FLAGS[mode] ?? [0]).map((f, i, all) => (i < all.length - 1 ? 0 : 0xff0000 | f));
}

/** Клавиша: x, ряд, ширина и высота — в ширинах обычной клавиши */
export type Key = [x: number, y: number, w: number, h: number];

/** Ряды из ширин клавиш; отрицательное число — промежуток */
function rows(spec: number[][]): Key[] {
  return spec.flatMap((widths, y) => {
    let x = 0;
    return widths.flatMap((w) => (w < 0 ? ((x -= w), []) : [[(x += w) - w, y, w, 1] as Key]));
  });
}

const ones = (n: number) => Array<number>(n).fill(1);

/** Сетка 15×4 для карточек режимов */
export const GRID = rows(Array.from({ length: 4 }, () => ones(15)));

/** Полноразмерная ANSI: основной блок, стрелки с навигацией, цифровой блок */
export const FULL: Key[] = [
  ...rows([
    [1, -1, ...ones(4), -0.5, ...ones(4), -0.5, ...ones(4), -0.25, ...ones(3)],
    [...ones(13), 2, -0.25, ...ones(3), -0.25, ...ones(4)],
    [1.5, ...ones(12), 1.5, -0.25, ...ones(3), -0.25, ...ones(3)],
    [1.75, ...ones(11), 2.25, -3.5, ...ones(3)],
    [2.25, ...ones(10), 2.75, -1.25, 1, -1.25, ...ones(3)],
    [1.25, 1.25, 1.25, 6.25, 1.25, 1.25, 1.25, 1.25, -0.25, ...ones(3), -0.25, 2, 1],
  ]),
  [21.5, 2, 1, 2], // «+» цифрового блока
  [21.5, 4, 1, 2], // Enter цифрового блока
];

const rand = (a: number) => {
  const s = Math.sin(a * 12.9898) * 43758.5453;
  return s - Math.floor(s);
};
const frac = (v: number) => v - Math.floor(v);
const clamp = (v: number) => Math.min(Math.max(v, 0), 1);

/** Что показывать: режим, палитра с флагами, ступени скорости и направления */
export type Shown = { mode: number; palette: number[]; speed?: number; direction?: number };

/** Цвет клавиши i в момент t (секунды) */
export function keyColor(keys: Key[], i: number, s: Shown, t: number): RGB {
  t *= (SPEEDS[s.speed ?? 3] ?? 100) / 100;
  const cols = Math.max(...keys.map(([x, , w]) => x + w));
  const nrows = Math.max(...keys.map(([, y]) => y)) + 1;
  const [kx, y, w] = keys[i];
  const x = kx + w / 2; // по центру клавиши
  const p0 = s.palette[0] ?? 0;
  const p1 = s.palette[1] ?? 0;
  const bg = rgb(p0);
  /** Цвет эффекта: с флагом радуги — оттенок по месту hue, со случайным — из таблицы по номеру срабатывания seed */
  const fg = (hue: number, seed: number): RGB => (p1 & HUE ? hsv(frac(hue), 1, 1) : p1 & RANDOM ? RANDOM_TAB[Math.floor(rand(seed) * RANDOM_TAB.length)] : rgb(p1));
  const on = (k: number, c: RGB) => mix(bg, c, clamp(k));

  /** Нажатия, придуманные для превью: каждые every секунд случайная клавиша; [центр, ряд, сколько секунд назад, номер клавиши, номер нажатия] */
  const presses = (n: number, every = 0.4) => {
    const now = Math.floor(t / every);
    return Array.from({ length: n }, (_, j) => {
      const k = Math.floor(rand(now - j) * keys.length);
      return [keys[k][0] + keys[k][2] / 2, keys[k][1], t - (now - j) * every, k, now - j] as const;
    });
  };
  /** Самое яркое из колец/пятен от нажатий: shape(расстояние, возраст) → яркость */
  const fromPresses = (shape: (d: number, age: number) => number, dist: (px: number, py: number) => number) => {
    let best: [number, RGB] = [0, bg];
    for (const [px, py, age, , n] of presses(2, 0.9)) {
      const k = shape(dist(px, py), age);
      if (k > best[0]) best = [k, fg(n * 0.13, n)];
    }
    return on(best[0], best[1]);
  };
  const round = (px: number, py: number) => Math.hypot(x - px, (y - py) * 1.2);

  switch (s.mode) {
    case 1: return p0 & HUE ? hsv(frac(t * 0.1), 1, 1) : bg; // с радугой вся клавиатура плавно меняет оттенок
    case 2: { // дыхание от фона к цвету; случайный цвет — новый на каждый вдох
      const cycle = (t * 2.1) / (2 * Math.PI);
      return on(0.5 - 0.5 * Math.cos(t * 2.1), fg(cycle * 0.1, Math.floor(cycle)));
    }
    case 3: { // полосы цвета бегут по фону в выбранную сторону; с радугой — переливы
      const d = s.direction ?? 0;
      const pos = d < 2 ? x / cols : (y + 0.5) / nrows;
      const v = (d === 1 || d === 3 ? pos : -pos) * (d < 2 ? 1 : 0.5) + t * 0.3;
      return p1 & HUE ? hsv(frac(v), 1, 1) : on(0.5 + 0.5 * Math.sin(v * 2 * Math.PI * 2), fg(0, 0));
    }
    case 4: { // капля падает по своему столбцу, хвост выше неё
      const col = Math.floor(x);
      const head = frac(t * 0.7 + rand(col)) * (nrows + 2) - 1;
      return on(head >= y ? 1 - (head - y) / 1.5 : 0, fg(col / cols, col));
    }
    case 5: { // огонёк бежит по каждому ряду, ряды со сдвигом
      const pos = (s.direction ?? 0) === 1 ? cols - x : x;
      const d = frac((t * 0.6 * cols - pos + y * 7 * 0.5) / (cols + 8)) * (cols + 8);
      return on(1 - d / 4, fg(x / cols + t * 0.1, 0));
    }
    case 6: return on(Math.max(0, Math.sin(2 * Math.PI * (t * (0.3 + rand(i) * 0.4) + rand(i + 7)))) ** 8, fg(rand(i + 3), i));
    case 7: { // нажатая клавиша вспыхивает и гаснет
      let best: [number, RGB] = [0, bg];
      for (const [, , age, k, n] of presses(3)) if (k === i && 1 - age / 1.2 > best[0]) best = [1 - age / 1.2, fg(n * 0.13, n)];
      return on(best[0], best[1]);
    }
    case 8: return fromPresses((d, age) => clamp(1 - Math.abs(d - age * 7) / 0.8) * clamp(1 - age / 1.8), round); // тонкое кольцо
    case 9: return fromPresses((d, age) => clamp(1 - Math.abs(d - age * 8) / 1.5) * clamp(1 - age / 1.8), (px, py) => (y === py ? Math.abs(x - px) : Math.abs(x - px) < 0.6 ? Math.abs(y - py) : Infinity));
    case 10: return fromPresses((d, age) => (d < age * 7 ? clamp(1 - (age * 7 - d) / 4) : 0) * clamp(1 - age / 1.8), round); // волна с хвостом
    case 11: return fromPresses((d, age) => clamp(1 - d / (1 + age * 5)) * clamp(1 - age / 1.8), round); // пятно расплывается от клавиши
    case 12: { // теплее там, где «чаще жмут»: левее середины
      const heat = clamp(0.9 - Math.hypot((x / cols - 0.27) / 0.47, ((y + 0.5) / nrows - 0.5) / 0.75) + 0.15 * Math.sin(t + rand(i) * 6));
      return mix(bg, rgb(p1), heat);
    }
    case 13: return hsv(rand(i), 0.9, 1);
    default: return bg;
  }
}

const DARK: RGB = [18, 20, 26];

/** Нарисовать клавиши: (x0, y0) — левый верхний угол, ux × uy — размер обычной клавиши, gapRow — отступ после первого ряда (в рядах) */
export function drawKeys(g: CanvasRenderingContext2D, keys: Key[], s: Shown, t: number, x0: number, y0: number, ux: number, uy: number, gapRow = 0, bright = 1) {
  const pad = Math.max(1.5, Math.min(ux, uy) * 0.06);
  keys.forEach(([x, y, w, h], i) => {
    // неподсвеченная клавиша не темнее корпуса
    g.fillStyle = css(scale(keyColor(keys, i, s, t), bright).map((v, j) => Math.max(v, DARK[j])) as RGB);
    g.beginPath();
    g.roundRect(x0 + x * ux + pad, y0 + (y + (y ? gapRow : 0)) * uy + pad, w * ux - pad * 2, h * uy - pad * 2, Math.max(2, Math.min(ux, uy) * 0.12));
    g.fill();
  });
}

/** Флаги цвета не трогаем, меняем только RGB */
export const withRgb = (c: number, [r, g, b]: RGB) => (c & FLAGS) | (r << 16) | (g << 8) | b;
