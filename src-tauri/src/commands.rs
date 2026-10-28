//! Команды для интерфейса — тонкие обёртки над devices, config и system.

use crate::devices::{Shared, asrock, zet};
use crate::{config, live, system};
use serde_json::Value;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

type Hw<'a> = State<'a, Shared>;

// --- плата ASRock

#[tauri::command(async)]
pub fn connect(hw: Hw) -> Result<Vec<asrock::ZoneView>, String> {
    let mut hw = hw.lock().unwrap();
    hw.board = None; // закрыть старый хэндл до повторного открытия
    let board = asrock::Board::open()?;
    let view = board.view();
    hw.board = Some(board);
    Ok(view)
}

#[tauri::command(async)]
pub fn write_zone(hw: Hw, id: usize, mode: u8, rgb: [u8; 3], speed: u8, palette: Vec<[u8; 3]>) -> Result<(), String> {
    let mut hw = hw.lock().unwrap();
    let show = hw.showing_user();
    hw.board(|b| b.set_zone(id, mode, rgb, speed, palette, show))
}

#[tauri::command(async)]
pub fn save(hw: Hw) -> Result<(), String> {
    hw.lock().unwrap().save()
}

// --- клавиатура ZET

#[tauri::command(async)]
pub fn kbd_open(hw: Hw, mode: u8) -> Result<Value, String> {
    let kbd = zet::Keyboard::open(mode)?;
    let view = serde_json::to_value(kbd.view()).unwrap();
    hw.lock().unwrap().kbd = Some(kbd);
    Ok(view)
}

#[tauri::command(async)]
pub fn kbd_effect(hw: Hw, mode: u8) -> Result<zet::Effect, String> {
    hw.lock().unwrap().keyboard(|k| k.read(mode))
}

#[tauri::command(async)]
pub fn kbd_set(hw: Hw, effect: zet::Effect) -> Result<(), String> {
    let mut hw = hw.lock().unwrap();
    let show = hw.showing_user();
    hw.keyboard(|k| k.set(effect, show))
}

// --- живые эффекты

#[tauri::command(async)]
pub fn set_live(hw: Hw, source: Option<live::Source>) -> Result<(), String> {
    hw.lock().unwrap().set_live(source)
}

// --- настройки и система

#[tauri::command(async)]
pub fn load_config() -> Result<Option<Value>, String> {
    config::load()
}

#[tauri::command(async)]
pub fn save_config(config: Value) -> Result<(), String> {
    config::save(&config)
}

#[tauri::command]
pub fn open_config_dir() -> Result<(), String> {
    config::open_dir()
}

#[tauri::command(async)]
pub fn system_info() -> Result<Value, String> {
    system::info()
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, on: bool) -> Result<(), String> {
    let a = app.autolaunch();
    if on { a.enable() } else { a.disable() }.map_err(|e| e.to_string())
}
