//! Клавиатура ZET Blade Pro напрямую по USB (протокол — protocol.rs).

mod protocol;

use super::Lights;
pub use protocol::Effect;
use protocol::{MODE_STATIC, ModeInfo, Zet};
use serde::Serialize;

pub struct Keyboard {
    link: Zet,
    effect: Effect, // что выбрал пользователь
    modes: Vec<ModeInfo>,
}

/// Клавиатура так, как её видит интерфейс
#[derive(Serialize)]
pub struct KeyboardView<'a> {
    modes: &'a [ModeInfo],
    effect: &'a Effect,
}

fn lost(e: hidapi::HidError) -> String {
    format!("Связь с клавиатурой ZET потеряна: {e}")
}

impl Keyboard {
    /// `mode` — какой режим считать включённым: как прочитать текущий, пока неизвестно
    pub fn open(mode: u8) -> Result<Self, String> {
        let mut link = Zet::connect().map_err(|e| format!("Клавиатура ZET недоступна: {e}"))?;
        let modes = link.modes().map_err(lost)?;
        let effect = link.effect(mode).map_err(lost)?;
        Ok(Self { link, effect, modes })
    }

    pub fn view(&self) -> KeyboardView<'_> {
        KeyboardView { modes: &self.modes, effect: &self.effect }
    }

    /// Настройки, которые клавиатура помнит для режима, — чтобы при смене режима показать его ползунки
    pub fn read(&mut self, mode: u8) -> Result<Effect, String> {
        self.link.effect(mode).map_err(lost)
    }

    /// Новый эффект; `show` — сразу вывести (иначе сейчас темнота или кадр эффекта)
    pub fn set(&mut self, effect: Effect, show: bool) -> Result<(), String> {
        self.effect = effect;
        if show { self.show(false) } else { Ok(()) }
    }

    /// Для `--profile`: включить эффект и сразу закрыть клавиатуру
    pub fn apply_once(effect: &Effect) -> Result<(), String> {
        Zet::connect().and_then(|mut z| z.set_effect(effect)).map_err(|e| format!("Клавиатура ZET: {e}"))
    }

    /// Свой цвет для «Музыки»; радуга и чёрный — белым
    fn base(&self) -> [u8; 3] {
        let c = self.effect.palette.first().copied().unwrap_or(0);
        let [flags, r, g, b] = c.to_be_bytes();
        if flags != 0 || c == 0 { [255; 3] } else { [r, g, b] }
    }

    fn static_color(&self, [r, g, b]: [u8; 3]) -> Effect {
        let brightness = self.modes.iter().find(|m| m.index == MODE_STATIC).map_or(6, |m| m.brights.saturating_sub(1));
        Effect { mode: MODE_STATIC, brightness, speed: 0, direction: 0, palette: vec![u32::from_be_bytes([0, r, g, b])] }
    }
}

impl Lights for Keyboard {
    fn show(&mut self, dark: bool) -> Result<(), String> {
        let e = if dark { Effect { mode: MODE_STATIC, palette: vec![0], ..self.effect.clone() } } else { self.effect.clone() };
        self.link.set_effect(&e).map_err(lost)
    }

    fn frame(&mut self, color: &dyn Fn([u8; 3]) -> [u8; 3]) -> Result<(), String> {
        let e = self.static_color(color(self.base()));
        self.link.set_effect(&e).map_err(lost)
    }
}
