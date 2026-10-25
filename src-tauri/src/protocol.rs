//! USB-контроллер подсветки ASRock Polychrome (26CE:01A2).
//!
//! Протокол восстановлен из AsrPolychromeRGB.exe 2.0.190 (функции 0x4abf14, 0x4ac160,
//! 0x4ac2e0, 0x4ac5b8, 0x4ac614) и проверен на B760M Pro RS/D4.
//! HID-отчёт 65 байт = [0x00, команда, 0x00, аргументы...]; ответ приходит без report ID.

use hidapi::{HidApi, HidDevice, HidResult};

const VID: u16 = 0x26CE;
const PID: u16 = 0x01A2;
const WRITE_ZONE: u8 = 0x10;
const READ_ZONE: u8 = 0x11;
const SAVE: u8 = 0x12;
const READ_CONFIG: u8 = 0x14;

pub const PALETTE_MODES: [u8; 2] = [16, 17];
/// 16 байт заголовка + 4 байта на цвет (+1 замыкающий) в 65-байтном пакете
pub const MAX_PALETTE: usize = 11;

/// Имена по теме ASRock; зона 7 на B760M Pro RS/D4 — подсветка самой платы (проверено глазами)
pub const ZONES: [&str; 8] = ["RGB 12V #1", "RGB 12V #2", "ARGB #1", "ARGB #2", "PCH", "Кожух I/O", "PCB", "Плата"];

pub fn packet(cmd: u8, args: &[u8]) -> [u8; 65] {
    let mut p = [0u8; 65];
    p[1] = cmd;
    p[3..3 + args.len()].copy_from_slice(args);
    p
}

/// Хвост пакета для палитровых режимов: точки градиента R,G,B,позиция% + замыкание на 100%.
pub fn palette_tail(colors: &[[u8; 3]], param: u8) -> Vec<u8> {
    let n = colors.len();
    let mut tail = vec![0, n as u8 + 1, param, 0, 0];
    for (i, c) in colors.iter().enumerate() {
        tail.extend(c);
        tail.push((i * 100 / n) as u8);
    }
    tail.extend(colors[0]);
    tail.push(100);
    tail
}

#[derive(Clone, PartialEq, Debug)]
pub struct Zone {
    pub mode: u8,
    pub rgb: [u8; 3],
    /// [задержка эффекта, неизвестный байт, синхронизация всех зон]
    pub extra: [u8; 3],
    pub palette: Vec<[u8; 3]>,
    palette_param: u8,
}

impl Zone {
    /// 0..=100, 100 — быстрее всего (байт — задержка, чем больше, тем медленнее)
    pub fn speed(&self) -> u8 {
        ((255 - self.extra[0]) as f32 / 2.55).round() as u8
    }

    pub fn set_speed(&mut self, speed: u8) {
        self.extra[0] = 255 - (speed as f32 * 2.55).round() as u8;
    }

    pub fn uses_palette(&self) -> bool {
        PALETTE_MODES.contains(&self.mode)
    }

    fn write_args(&self, zone: usize) -> Vec<u8> {
        // 0 в байте синхронизации: иначе контроллер разносит запись по всем зонам
        let mut args = vec![zone as u8, self.mode, self.rgb[0], self.rgb[1], self.rgb[2], self.extra[0], self.extra[1], 0];
        if self.uses_palette() && !self.palette.is_empty() {
            args.extend(palette_tail(&self.palette, self.palette_param));
        }
        args
    }
}

pub struct Controller {
    _api: HidApi,
    dev: HidDevice,
}

impl Controller {
    pub fn open() -> HidResult<Self> {
        let api = HidApi::new()?;
        let dev = api.open(VID, PID)?;
        Ok(Self { _api: api, dev })
    }

    fn send(&self, cmd: u8, args: &[u8]) -> HidResult<[u8; 64]> {
        self.dev.write(&packet(cmd, args))?;
        let mut reply = [0u8; 64];
        self.dev.read_timeout(&mut reply, 500)?; // контроллер отвечает на каждую команду
        Ok(reply)
    }

    pub fn active_zones(&self) -> HidResult<Vec<usize>> {
        let mask = self.send(READ_CONFIG, &[1])?[4];
        Ok((0..8).filter(|z| mask >> z & 1 == 1).collect())
    }

    pub fn read_zone(&self, zone: usize) -> HidResult<Zone> {
        let r = self.send(READ_ZONE, &[zone as u8])?;
        let p = self.send(READ_CONFIG, &[10, zone as u8])?;
        let n = (p[4] as usize).saturating_sub(1).min(MAX_PALETTE);
        let mut palette: Vec<[u8; 3]> = (0..n).map(|i| [p[8 + 4 * i], p[9 + 4 * i], p[10 + 4 * i]]).collect();
        if palette.is_empty() {
            palette = vec![[255, 0, 0], [0, 255, 0], [0, 0, 255]];
        }
        Ok(Zone { mode: r[4], rgb: [r[5], r[6], r[7]], extra: [r[8], r[9], r[10]], palette, palette_param: p[5] })
    }

    pub fn write_zone(&self, zone: usize, z: &Zone) -> HidResult<()> {
        self.send(WRITE_ZONE, &z.write_args(zone)).map(|_| ())
    }

    /// Запомнить текущие настройки в контроллере (как AsrPolychromeRGB после записи)
    pub fn save(&self) -> HidResult<()> {
        self.send(SAVE, &[]).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_matches_asrock() {
        // Команда, которую AsrPolychromeRGB.exe шлёт через AsrISP.exe:
        // /CMD 0x00 0x10 0x00 0x00 0x01 0x00 0x33 0x99 0xD0 0xFF 0x01 0x00
        let asrock = [0x00, 0x10, 0x00, 0x00, 0x01, 0x00, 0x33, 0x99, 0xD0, 0xFF, 0x01, 0x00];
        let p = packet(0x10, &[0, 1, 0x00, 0x33, 0x99, 0xD0, 0xFF, 0x01]);
        assert_eq!(&p[..12], &asrock);
        assert!(p[12..].iter().all(|&b| b == 0));
    }

    #[test]
    fn palette_matches_stored_format() {
        // Так палитру хранит контроллер (ответ на 14 0a): 04 0a 00 00 | ff0000@0 00ff00@33 0000ff@66 ff0000@100
        let tail = palette_tail(&[[255, 0, 0], [0, 255, 0], [0, 0, 255]], 0x0a);
        let stored = [4, 0x0a, 0, 0, 255, 0, 0, 0, 0, 255, 0, 33, 0, 0, 255, 66, 255, 0, 0, 100];
        assert_eq!(tail[0], 0);
        assert_eq!(&tail[1..], &stored);
    }

    #[test]
    fn longest_palette_fits_packet() {
        let z = Zone { mode: 16, rgb: [0; 3], extra: [0x70, 0xff, 0], palette: vec![[1, 2, 3]; MAX_PALETTE], palette_param: 10 };
        packet(WRITE_ZONE, &z.write_args(2)); // паникует, если не влезает
    }

    #[test]
    fn speed_roundtrip_and_direction() {
        let mut z = Zone { mode: 2, rgb: [0; 3], extra: [0x70, 0xff, 0], palette: vec![], palette_param: 0 };
        for s in 0..=100 {
            z.set_speed(s);
            assert_eq!(z.speed(), s);
        }
        z.set_speed(100);
        assert_eq!(z.extra[0], 0); // проверено глазами: 0x10 мигает быстрее, чем 0xF0
    }
}
