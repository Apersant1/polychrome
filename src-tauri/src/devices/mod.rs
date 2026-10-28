//! Устройства подсветки и то, что на них должно гореть прямо сейчас.
//!
//! Настройки пользователя хранит каждое устройство само. `Hw` накладывает поверх них
//! темноту (idle.rs) и кадры живых эффектов (live.rs) и следит, чтобы ни то ни другое
//! не попало в память устройств.

pub mod asrock;
pub mod zet;

use crate::live::Source;
use std::sync::Mutex;

/// Общее для всех устройств подсветки
pub trait Lights {
    /// Показать настройки пользователя или темноту
    fn show(&mut self, dark: bool) -> Result<(), String>;
    /// Кадр живого эффекта статичным цветом; `color` получает собственный цвет устройства (зоны, клавиатуры)
    fn frame(&mut self, color: &dyn Fn([u8; 3]) -> [u8; 3]) -> Result<(), String>;
}

#[derive(Default)]
pub struct Hw {
    pub board: Option<asrock::Board>,
    pub kbd: Option<zet::Keyboard>,
    dark: u8,             // причины из idle: пока не 0, на всех устройствах темнота
    live: Option<Source>, // пока задан, на устройствах кадры live, а не настройки пользователя
}

pub type Shared = Mutex<Hw>;

impl Hw {
    /// На железе сейчас настройки пользователя, а не темнота или кадры эффекта
    pub fn showing_user(&self) -> bool {
        self.dark == 0 && self.live.is_none()
    }

    /// Действие с платой; при ошибке связи плату отключаем — интерфейс предложит переподключить
    pub fn board<T>(&mut self, f: impl FnOnce(&mut asrock::Board) -> Result<T, String>) -> Result<T, String> {
        let b = self.board.as_mut().ok_or("Нет связи с контроллером")?;
        let r = f(b);
        if r.is_err() {
            self.board = None;
        }
        r
    }

    /// Действие с клавиатурой; при ошибке связи отключаем её, панель клавиатуры скроется
    pub fn keyboard<T>(&mut self, f: impl FnOnce(&mut zet::Keyboard) -> Result<T, String>) -> Result<T, String> {
        let k = self.kbd.as_mut().ok_or("Клавиатура не подключена")?;
        let r = f(k);
        if r.is_err() {
            self.kbd = None;
        }
        r
    }

    /// Применить ко всем устройствам; ошибка платы возвращается, клавиатура необязательна
    fn each(&mut self, f: impl Fn(&mut dyn Lights) -> Result<(), String>) -> Result<(), String> {
        let res = if self.board.is_some() { self.board(|b| f(b)) } else { Ok(()) };
        if self.kbd.is_some() {
            let _ = self.keyboard(|k| f(k));
        }
        res
    }

    /// Вывести настройки пользователя или темноту; во время живого эффекта кадры рисует live.rs
    pub fn refresh(&mut self) -> Result<(), String> {
        if self.live.is_some() && self.dark == 0 {
            return Ok(());
        }
        let dark = self.dark != 0;
        self.each(|d| d.show(dark))
    }

    pub fn set_dark(&mut self, why: u8, started: bool) {
        let was = self.dark != 0;
        self.dark = if started { self.dark | why } else { self.dark & !why };
        if was != (self.dark != 0) {
            let _ = self.refresh(); // плата при ошибке отключена, интерфейс увидит это при следующей записи
        }
    }

    pub fn set_live(&mut self, src: Option<Source>) -> Result<(), String> {
        self.live = src;
        if src.is_none() { self.refresh() } else { Ok(()) }
    }

    /// Источник кадров, если их сейчас надо рисовать
    pub fn live_source(&self) -> Option<Source> {
        if self.dark != 0 || (self.board.is_none() && self.kbd.is_none()) { None } else { self.live }
    }

    /// Кадр живого эффекта, если источник ещё тот же: его могли выключить, пока считали кадр
    pub fn frame(&mut self, src: Source, color: impl Fn([u8; 3]) -> [u8; 3]) {
        if self.live_source() == Some(src) {
            let _ = self.each(|d| d.frame(&color));
        }
    }

    /// Запомнить зоны в контроллере: на железе в этот момент должны быть они, а не темнота или кадр
    pub fn save(&mut self) -> Result<(), String> {
        let covered = !self.showing_user();
        self.board(|b| {
            if covered {
                b.show(false)?;
            }
            b.save()
        })?;
        if covered {
            let _ = self.refresh(); // вернуть темноту; кадры эффекта live.rs дорисует сам
        }
        Ok(())
    }
}
