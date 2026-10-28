// Железо из WMI (src-tauri/src/system.rs) — для устройств, которыми пока не управляем.

import { api } from '$lib/api';
import { app } from './app.svelte';

export async function loadSystem() {
  try {
    app.sys = await api.systemInfo();
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
