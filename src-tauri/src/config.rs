//! config.json рядом с exe: профили, имена зон, настройки клавиатуры. Программа переносится вместе с ним.
//! Содержимое собирает интерфейс (app.svelte.ts), здесь — только надёжное чтение и запись.

use serde_json::Value;
use std::{fs, path::PathBuf};

fn path() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    exe.parent().map_or_else(|| PathBuf::from("config.json"), |dir| dir.join("config.json"))
}

/// None — файла ещё нет (первый запуск)
pub fn load() -> Result<Option<Value>, String> {
    let path = path();
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Не удалось прочитать {}: {e}", path.display())),
    };
    match serde_json::from_slice(&bytes) {
        Ok(v) => Ok(Some(v)),
        Err(_) => {
            // испорченный файл не затираем молча, а откладываем рядом
            fs::rename(&path, path.with_extension("json.bad")).map_err(|e| e.to_string())?;
            Ok(None)
        }
    }
}

pub fn save(config: &Value) -> Result<(), String> {
    let path = path();
    let tmp = path.with_extension("json.tmp"); // через временный файл: обрыв записи не портит старый
    fs::create_dir_all(path.parent().unwrap())
        .and_then(|_| fs::write(&tmp, serde_json::to_vec_pretty(config).unwrap()))
        .and_then(|_| fs::rename(&tmp, &path))
        .map_err(|e| format!("Не удалось сохранить настройки: {e}"))
}

pub fn open_dir() -> Result<(), String> {
    let dir = path().parent().unwrap().to_owned();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::process::Command::new("explorer").arg(dir).spawn().map(|_| ()).map_err(|e| e.to_string())
}
