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

pub const MODE_OFF: u8 = 0;
pub const MODE_STATIC: u8 = 1;
pub const PALETTE_MODES: [u8; 2] = [16, 17];
/// 16 байт заголовка + 4 байта на цвет (+1 замыкающий) в 65-байтном пакете
pub const MAX_PALETTE: usize = 11;

/// Имена по теме ASRock. На B760M Pro RS/D4 (ответ 14 02 — светодиодов на зону: 1 1 80 80 30 30 3 80)
/// зона 6 — 3 диода в уголке платы справа сверху, зона 7 — третий разъём ARGB
pub const ZONES: [&str; 8] = ["RGB 12V #1", "RGB 12V #2", "ARGB #1", "ARGB #2", "PCH", "Кожух I/O", "Плата", "ARGB #3"];

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

    /// Настройки из интерфейса или профиля; палитру недопустимой длины пропускаем
    pub fn set(&mut self, mode: u8, rgb: [u8; 3], speed: u8, palette: Vec<[u8; 3]>) {
        self.mode = mode;
        self.rgb = rgb;
        self.set_speed(speed.min(100));
        if (2..=MAX_PALETTE).contains(&palette.len()) {
            self.palette = palette;
        }
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

    /// Светодиодов на каждом из 8 каналов (ответ на 14 02)
    #[allow(dead_code)] // прямой режим по диодам проверен на железе, ждёт Ambilight по диодам
    pub fn led_counts(&self) -> HidResult<[u8; 8]> {
        let r = self.send(READ_CONFIG, &[2])?;
        Ok(r[4..12].try_into().unwrap())
    }

    #[allow(dead_code)]
    /// Цвета всех диодов всех каналов подряд, в порядке каналов (длины — led_counts).
    /// Проверено на B760M Pro RS/D4 (direct_stream_demo).
    pub fn write_leds(&self, colors: &[[u8; 3]]) -> HidResult<()> {
        for p in led_packets(colors) {
            self.dev.write(&p)?;
            // AsrPolychromeRGB ответ здесь не читает; вычерпываем, если он есть, чтобы не сбить ответы другим командам
            let mut reply = [0u8; 64];
            while self.dev.read_timeout(&mut reply, 0)? > 0 {}
        }
        Ok(())
    }
}

#[allow(dead_code)]
/// Поток цветов по диодам — как FUN_004acbcc в AsrPolychromeRGB.exe 2.0.191:
/// `10 00 FF E3 00 00 <всего, LE16>` + 18 цветов, затем `10 00 FF E4` + по 19 цветов
pub fn led_packets(colors: &[[u8; 3]]) -> Vec<[u8; 65]> {
    let n = colors.len() as u16;
    let (first, rest) = colors.split_at(colors.len().min(18));
    let mut head = vec![0xFF, 0xE3, 0, 0, n as u8, (n >> 8) as u8];
    head.extend(first.iter().flatten());
    let mut out = vec![packet(WRITE_ZONE, &head)];
    for chunk in rest.chunks(19) {
        let mut args = vec![0xFF, 0xE4];
        args.extend(chunk.iter().flatten());
        out.push(packet(WRITE_ZONE, &args));
    }
    out
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
    fn led_packets_layout() {
        let colors: Vec<[u8; 3]> = (0..305u16).map(|i| [i as u8, 1, 2]).collect();
        let ps = led_packets(&colors);
        assert_eq!(ps.len(), 1 + (305 - 18usize).div_ceil(19));
        assert_eq!(&ps[0][..12], &[0, 0x10, 0, 0xFF, 0xE3, 0, 0, 305u16 as u8, 1, 0, 1, 2]); // 305 = 0x131
        assert_eq!(&ps[0][60..63], &[17, 1, 2]); // 18-й цвет
        assert_eq!(&ps[1][..8], &[0, 0x10, 0, 0xFF, 0xE4, 18, 1, 2]);
        assert_eq!(&ps[1][59..62], &[36, 1, 2]); // 19-й цвет второго пакета
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

/// Нужен контроллер: `cargo test --release -- --ignored --nocapture exchange_time`.
/// Пишет в зону её же настройки — на подсветке ничего не меняется.
#[cfg(test)]
mod hw_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore]
    fn exchange_time() {
        let c = Controller::open().unwrap();
        let zones = c.active_zones().unwrap();
        let z = c.read_zone(zones[0]).unwrap();
        let n = 200;
        let t = Instant::now();
        for _ in 0..n {
            c.write_zone(zones[0], &z).unwrap();
        }
        let per = t.elapsed() / n;
        println!("запись зоны: {per:?} на команду, {} зон за кадр → до {:.0} кадров/с", zones.len(), 1.0 / (per.as_secs_f64() * zones.len() as f64));
    }

    /// Смотреть на подсветку: 2 с красный, 2 с зелёный, 2 с синий, 5 с бегущая белая точка по каждому каналу,
    /// потом зоны возвращаются как были. В память контроллера ничего не пишется.
    #[test]
    #[ignore]
    fn direct_stream_demo() {
        use std::{thread::sleep, time::Duration};
        let c = Controller::open().unwrap();
        let counts = c.led_counts().unwrap();
        let total: usize = counts.iter().map(|&n| n as usize).sum();
        println!("диодов на каналах: {counts:?}, всего {total}");
        let saved: Vec<_> = c.active_zones().unwrap().into_iter().map(|z| (z, c.read_zone(z).unwrap())).collect();
        let t = Instant::now();
        c.write_leds(&vec![[255, 0, 0]; total]).unwrap();
        println!("кадр из {total} диодов: {:?}", t.elapsed());
        sleep(Duration::from_secs(2));
        c.write_leds(&vec![[0, 255, 0]; total]).unwrap();
        sleep(Duration::from_secs(2));
        c.write_leds(&vec![[0, 0, 255]; total]).unwrap();
        sleep(Duration::from_secs(2));
        for step in 0..150 {
            let mut frame = Vec::with_capacity(total);
            for &n in &counts {
                frame.extend((0..n as usize).map(|i| if n > 0 && i == step % n as usize { [255, 255, 255] } else { [0, 0, 0] }));
            }
            c.write_leds(&frame).unwrap();
            sleep(Duration::from_millis(33));
        }
        for (z, zone) in &saved {
            c.write_zone(*z, zone).unwrap();
        }
    }
}
