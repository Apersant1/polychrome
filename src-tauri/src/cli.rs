//! `Polychrome.exe --profile Имя`: применить профиль, записать в контроллер и выйти — для Планировщика задач.

use crate::config;
use crate::devices::{asrock::Board, zet};
use serde::Deserialize;
use serde_json::Value;

/// Настройки зоны в профиле — как `Look` в leds.ts: цвет хранится на полной яркости
#[derive(Deserialize)]
struct Look {
    mode: u8,
    color: [u8; 3],
    bright: u8,
    speed: u8,
    palette: Vec<[u8; 3]>,
}

impl Look {
    /// Как `actual()` в leds.ts: цвет, приглушённый яркостью
    fn dim(&self, c: [u8; 3]) -> [u8; 3] {
        let k = (self.bright as f64 / 100.0).clamp(0.0, 1.0);
        c.map(|v| (v as f64 * k).round() as u8)
    }
}

pub fn apply_profile(name: &str) -> Result<(), String> {
    let config = config::load()?.ok_or("Нет файла настроек — запустите программу один раз")?;
    let profile = config["profiles"].as_array().into_iter().flatten().find(|p| p["name"] == name).ok_or_else(|| format!("Нет профиля «{name}»"))?;
    let mut board = Board::open()?;
    for id in board.zone_ids() {
        // ключ — номер зоны или '*' для всех, как profileLook() в профилях интерфейса
        let zones = &profile["zones"];
        let l = match &zones[id.to_string()] {
            Value::Null => &zones["*"],
            l => l,
        };
        if l.is_null() {
            continue;
        }
        let l = Look::deserialize(l).map_err(|e| format!("Профиль «{name}», зона {id}: {e}"))?;
        board.set_zone(id, l.mode, l.dim(l.color), l.speed, l.palette.iter().map(|&p| l.dim(p)).collect(), true)?;
    }
    board.save()?;
    if let Some(k) = profile.get("kbd").filter(|k| !k.is_null()) {
        let e = zet::Effect::deserialize(k).map_err(|e| format!("Профиль «{name}», клавиатура: {e}"))?;
        zet::Keyboard::apply_once(&e)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_from_profile_dims_like_frontend() {
        // так профиль пишет saveProfile(); actual() в leds.ts даёт Math.round(v * 0.5)
        let l: Look = serde_json::from_str(r#"{"mode":16,"color":[255,128,1],"bright":50,"speed":70,"palette":[[255,0,0],[0,0,254]]}"#).unwrap();
        assert_eq!(l.dim(l.color), [128, 64, 1]);
        assert_eq!(l.dim(l.palette[1]), [0, 0, 127]);
    }
}
