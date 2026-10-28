// Все вызовы бэкенда (src-tauri/src/commands.rs) в одном месте и с типами.

import { invoke } from '@tauri-apps/api/core';
import type { RGB } from './leds';

/** Зона платы так, как её отдаёт контроллер: цвет уже приглушён яркостью */
export type ZoneDto = { id: number; name: string; mode: number; rgb: RGB; speed: number; palette: RGB[] };
/** Клавиатура ZET: режим 1 статичный … 13 своя раскраска; цвет 0xRRGGBB, бит 24 — радуга */
export type KbdEffect = { mode: number; brightness: number; speed: number; direction: number; palette: number[] };
export type KbdMode = { index: number; speeds: number; brights: number; dirs: number; colors: number };
export type KbdView = { modes: KbdMode[]; effect: KbdEffect };
export type Live = 'music' | 'screen';
export type SysInfo = {
  board?: { Manufacturer: string; Product: string };
  cpu?: { Name: string }[];
  gpu?: { Name: string; DriverVersion: string }[];
  ram?: { Manufacturer: string; PartNumber: string; Capacity: number; ConfiguredClockSpeed: number; DeviceLocator: string }[];
};

export const api = {
  connect: () => invoke<ZoneDto[]>('connect'),
  writeZone: (id: number, mode: number, speed: number, rgb: RGB, palette: RGB[]) => invoke('write_zone', { id, mode, speed, rgb, palette }),
  save: () => invoke('save'),

  kbdOpen: (mode: number) => invoke<KbdView>('kbd_open', { mode }),
  kbdEffect: (mode: number) => invoke<KbdEffect>('kbd_effect', { mode }),
  kbdSet: (effect: KbdEffect) => invoke('kbd_set', { effect }),

  setLive: (source: Live | null) => invoke('set_live', { source }),

  loadConfig: <T>() => invoke<T | null>('load_config'),
  saveConfig: (config: unknown) => invoke('save_config', { config }),
  openConfigDir: () => invoke('open_config_dir'),
  systemInfo: () => invoke<SysInfo>('system_info'),
  getAutostart: () => invoke<boolean>('get_autostart'),
  setAutostart: (on: boolean) => invoke('set_autostart', { on }),
};
