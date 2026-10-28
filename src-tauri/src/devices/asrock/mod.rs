//! Подсветка платы ASRock через USB-контроллер Polychrome (протокол — protocol.rs).

mod protocol;

use super::Lights;
use hidapi::{HidError, HidResult};
use protocol::{Controller, MODE_OFF, MODE_STATIC, ZONES, Zone};
use serde::Serialize;

/// Зона так, как её видит интерфейс
#[derive(Serialize)]
pub struct ZoneView {
    id: usize,
    name: &'static str,
    mode: u8,
    rgb: [u8; 3],
    speed: u8,
    palette: Vec<[u8; 3]>,
}

pub struct Board {
    ctrl: Controller,
    zones: Vec<(usize, Zone)>, // что выбрал пользователь, даже пока на железе темнота или кадр эффекта
    pub dirty: bool,           // на железе есть то, чего нет в памяти контроллера
}

fn lost(e: HidError) -> String {
    format!("Связь с контроллером потеряна: {e}")
}

impl Board {
    pub fn open() -> Result<Self, String> {
        let ctrl = Controller::open().map_err(|e| format!("Контроллер ASRock не найден: {e}"))?;
        let zones = ctrl
            .active_zones()
            .and_then(|ids| ids.into_iter().map(|z| ctrl.read_zone(z).map(|zone| (z, zone))).collect::<HidResult<Vec<_>>>())
            .map_err(|e| format!("Не удалось прочитать зоны: {e}"))?;
        Ok(Self { ctrl, zones, dirty: false })
    }

    pub fn view(&self) -> Vec<ZoneView> {
        self.zones.iter().map(|(id, z)| ZoneView { id: *id, name: ZONES[*id], mode: z.mode, rgb: z.rgb, speed: z.speed(), palette: z.palette.clone() }).collect()
    }

    pub fn zone_ids(&self) -> Vec<usize> {
        self.zones.iter().map(|(id, _)| *id).collect()
    }

    /// Новые настройки зоны; `show` — сразу вывести на железо (иначе там сейчас темнота или кадр эффекта)
    pub fn set_zone(&mut self, id: usize, mode: u8, rgb: [u8; 3], speed: u8, palette: Vec<[u8; 3]>, show: bool) -> Result<(), String> {
        let Some((_, z)) = self.zones.iter_mut().find(|(z, _)| *z == id) else { return Err(format!("Нет зоны {id}")) };
        z.set(mode, rgb, speed, palette);
        self.dirty = true;
        if show {
            self.ctrl.write_zone(id, z).map_err(lost)?;
        }
        Ok(())
    }

    /// Запомнить в контроллере то, что сейчас на железе (как AsrPolychromeRGB после записи)
    pub fn save(&mut self) -> Result<(), String> {
        self.ctrl.save().map_err(lost)?;
        self.dirty = false;
        Ok(())
    }

    fn paint(&self, dark: bool) -> HidResult<()> {
        self.zones.iter().try_for_each(|(id, z)| {
            let mut z = z.clone();
            if dark {
                z.mode = MODE_OFF;
            }
            self.ctrl.write_zone(*id, &z)
        })
    }
}

impl Lights for Board {
    fn show(&mut self, dark: bool) -> Result<(), String> {
        if self.paint(dark).is_ok() {
            return Ok(());
        }
        // после сна старый хэндл бывает мёртв — пробуем открыть заново
        self.ctrl = Controller::open().map_err(lost)?;
        self.paint(dark).map_err(lost)
    }

    /// Зоны, которые пользователь не выключил, — статичным цветом
    fn frame(&mut self, color: &dyn Fn([u8; 3]) -> [u8; 3]) -> Result<(), String> {
        for (id, z) in self.zones.iter().filter(|(_, z)| z.mode != MODE_OFF) {
            let mut f = z.clone();
            f.mode = MODE_STATIC;
            f.rgb = color(z.rgb);
            self.ctrl.write_zone(*id, &f).map_err(lost)?;
        }
        Ok(())
    }
}
