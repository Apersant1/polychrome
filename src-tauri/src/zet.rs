//! Клавиатура ZET Blade Pro (0416:A0F8, прошивка Whiftec KB100) напрямую по USB, без Blade Pro Software.
//! 32-байтные отчёты ID 6 в коллекцию COL06 — как их шлёт wkbsvc.exe 1.2.4.2 (снято Ghidra):
//! `06 BE <команда> …`, клавиатура отвечает тем же пакетом с 7D вместо BE.

use hidapi::{HidApi, HidDevice, HidResult};
use serde::{Deserialize, Serialize};

const VID: u16 = 0x0416;
const PID: u16 = 0xA0F8;
const REPORT: u8 = 0x06;
const REQ: u8 = 0xBE;
const ACK: u8 = 0x7D;
const GET_EFFECT: u8 = 0x14;
const SET_EFFECT: u8 = 0x15;
const KEYBOARD: u8 = 1; // устройство 2 — видимо, боковая подсветка
const PROFILE: u8 = 1;

pub const MODE_STATIC: u8 = 1;

/// mode_id режимов 1…13, как их отдаёт служба ZET (команда 0x1f); одна таблица на модель
const MODE_IDS: [u32; 13] = [0x17701, 0x27702, 0x37742, 0x47702, 0x57722, 0x67702, 0x80077702, 0x80087702, 0x80097702, 0x800a7702, 0x800b7702, 0xd0702, 0xe0801];

/// Эффект клавиатуры: режим 1 статичный … 13 своя раскраска;
/// цвет 0xRRGGBB, в старшем байте флаги (бит 24 — радуга, бит 26 — случайный)
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Effect {
    pub mode: u8,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8,
    pub palette: Vec<u32>,
}

/// Сколько ступеней у ползунков режима и цветов в его палитре
#[derive(Serialize, PartialEq, Debug)]
pub struct ModeInfo {
    pub index: u8,
    pub speeds: u8,
    pub brights: u8,
    pub dirs: u8,
    pub colors: u8,
}

pub struct Zet {
    _api: HidApi,
    dev: HidDevice,
}

impl Zet {
    pub fn connect() -> HidResult<Self> {
        let api = HidApi::new()?;
        // COL06: usage page 1, usage 0 на интерфейсе 1 — единственная такая коллекция у клавиатуры
        let info = api
            .device_list()
            .find(|d| d.vendor_id() == VID && d.product_id() == PID && d.interface_number() == 1 && d.usage_page() == 1 && d.usage() == 0)
            .ok_or_else(|| hidapi::HidError::HidApiError { message: "клавиатура ZET не подключена".into() })?;
        let dev = info.open_device(&api)?;
        Ok(Self { _api: api, dev })
    }

    pub fn modes(&mut self) -> HidResult<Vec<ModeInfo>> {
        Ok(MODE_IDS.iter().zip(1..).map(|(&id, index)| mode_info(index, id)).collect())
    }

    /// Настройки, сохранённые в клавиатуре для режима (не обязательно включённого)
    pub fn effect(&mut self, mode: u8) -> HidResult<Effect> {
        let mut p = [0u8; 32];
        p[..3].copy_from_slice(&[REPORT, REQ, GET_EFFECT]);
        p[4] = KEYBOARD;
        p[6] = mode;
        let r = self.call(&p)?;
        Ok(parse(&r))
    }

    /// Включить режим с этими настройками; прошивка сохраняет их сама
    pub fn set_effect(&mut self, e: &Effect) -> HidResult<()> {
        self.call(&packet(e)).map(|_| ())
    }

    /// Отправить и дождаться подтверждения той же команды (посторонние отчёты пропускаем)
    fn call(&self, p: &[u8; 32]) -> HidResult<[u8; 32]> {
        self.dev.write(p)?;
        let mut r = [0u8; 32];
        for _ in 0..8 {
            if self.dev.read_timeout(&mut r, 300)? == 0 {
                break;
            }
            if r[1] == ACK && r[2] == p[2] {
                return Ok(r);
            }
        }
        Err(hidapi::HidError::HidApiError { message: "клавиатура ZET не ответила".into() })
    }
}

/// `06 BE 15 00 <устройство> <профиль> <режим> <скорость> <яркость> <направление> 00 00` + до 4 цветов R, G, B, флаги
fn packet(e: &Effect) -> [u8; 32] {
    let mut p = [0u8; 32];
    p[..3].copy_from_slice(&[REPORT, REQ, SET_EFFECT]);
    p[4..10].copy_from_slice(&[KEYBOARD, PROFILE, e.mode, e.speed, e.brightness, e.direction]);
    for (i, c) in e.palette.iter().take(4).enumerate() {
        let [flags, r, g, b] = c.to_be_bytes();
        p[12 + 4 * i..16 + 4 * i].copy_from_slice(&[r, g, b, flags]);
    }
    p
}

fn parse(r: &[u8; 32]) -> Effect {
    let palette = r[12..28].chunks(4).map(|c| u32::from_be_bytes([c[3], c[0], c[1], c[2]])).collect();
    Effect { mode: r[6], speed: r[7], brightness: r[8], direction: r[9], palette }
}

/// Младшие 16 бит mode_id — по полубайту: ступени скорости, яркости, направлений и число цветов (как KbApi.modeInfoFromId)
fn mode_info(index: u8, id: u32) -> ModeInfo {
    ModeInfo { index, speeds: (id >> 12 & 15) as u8, brights: (id >> 8 & 15) as u8, dirs: (id >> 4 & 15) as u8, colors: (id & 15) as u8 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_matches_wkbsvc() {
        // режим 4, яркость 5, цвета чёрный и красный с флагом радуги — так его включает служба ZET
        let e = Effect { mode: 4, brightness: 5, speed: 0, direction: 0, palette: vec![0, 0x01FF0000] };
        let p = packet(&e);
        assert_eq!(&p[..20], &[0x06, 0xBE, 0x15, 0, 1, 1, 4, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0, 0, 1]);
        // ответ клавиатуры на чтение — тот же пакет с 7D 14
        let mut r = p;
        r[1..3].copy_from_slice(&[ACK, GET_EFFECT]);
        assert_eq!(parse(&r).palette[..2], e.palette[..]);
        assert_eq!(parse(&r).brightness, 5);
    }

    #[test]
    fn mode_ids_from_service() {
        assert_eq!(mode_info(1, MODE_IDS[0]), ModeInfo { index: 1, speeds: 7, brights: 7, dirs: 0, colors: 1 }); // статичный
        assert_eq!(mode_info(3, MODE_IDS[2]), ModeInfo { index: 3, speeds: 7, brights: 7, dirs: 4, colors: 2 }); // поток
        assert_eq!(mode_info(13, MODE_IDS[12]), ModeInfo { index: 13, speeds: 0, brights: 8, dirs: 0, colors: 1 }); // своя раскраска
    }

    #[test]
    #[ignore] // нужна клавиатура и закрытая Blade Pro Software: cargo test -- --ignored --nocapture zet_read
    fn zet_read() {
        let mut z = Zet::connect().unwrap();
        println!("{:?}", z.effect(4).unwrap());
    }
}
