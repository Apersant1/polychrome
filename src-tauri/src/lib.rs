//! Polychrome: своя программа подсветки вместо AsrPolychromeRGB и Blade Pro Software.
//!
//! - `devices` — устройства (плата ASRock, клавиатура ZET) и то, что на них горит сейчас;
//! - `live`, `idle` — живые эффекты и гашение при блокировке/сне поверх настроек пользователя;
//! - `commands` — команды для интерфейса; `config`, `system`, `cli` — файл настроек, WMI, `--profile`;
//! - здесь — запуск: плагины, трей, горячие клавиши, выход.

mod cli;
mod commands;
mod config;
mod devices;
mod idle;
mod live;
mod system;

use devices::Shared;
use tauri::{
    App, AppHandle, Emitter, Manager, RunEvent, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Глобальные клавиши → событие `hotkey` в интерфейс (работают, пока программа запущена)
const HOTKEYS: [(&str, &str); 2] = [("Ctrl+Alt+L", "toggle"), ("Ctrl+Alt+P", "next")];

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let [flag, name] = &args[..]
        && flag == "--profile"
    {
        let code = cli::apply_profile(name).map_or_else(|e| { eprintln!("{e}"); 1 }, |()| 0);
        std::process::exit(code);
    }

    tauri::Builder::default()
        // первым: второй запуск exe только показывает окно уже работающей копии
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show_window(app)))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--tray"])))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    if let Some((_, action)) = HOTKEYS.iter().find(|(k, _)| k.parse::<Shortcut>().is_ok_and(|k| k == *shortcut)) {
                        let _ = app.emit("hotkey", action);
                    }
                })
                .build(),
        )
        .manage(Shared::default())
        .setup(|app| {
            for (keys, _) in HOTKEYS {
                // сочетание может быть занято другой программой — работаем без него
                if let Err(e) = app.global_shortcut().register(keys) {
                    eprintln!("{keys}: {e}");
                }
            }
            tray(app)?;
            // автозапуск передаёт --tray: стартуем без окна (в tauri.conf.json окно скрыто)
            if !std::env::args().any(|a| a == "--tray") {
                show_window(app.handle());
            }
            live::run(app.handle().clone());
            let handle = app.handle().clone();
            idle::watch(move |why, started| handle.state::<Shared>().lock().unwrap().set_dark(why, started));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::write_zone,
            commands::save,
            commands::kbd_open,
            commands::kbd_effect,
            commands::kbd_set,
            commands::set_live,
            commands::load_config,
            commands::save_config,
            commands::open_config_dir,
            commands::system_info,
            commands::get_autostart,
            commands::set_autostart,
        ])
        .on_window_event(|window, event| {
            // крестик прячет в трей, выход — через меню трея
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                let state = app.state::<Shared>();
                let mut hw = state.lock().unwrap();
                if hw.board.as_ref().is_some_and(|b| b.dirty) {
                    let _ = hw.save(); // не потерять настройку при выходе
                }
            }
        });
}

fn tray(app: &App) -> tauri::Result<()> {
    let menu = Menu::with_items(app, &[&MenuItem::with_id(app, "show", "Открыть", true, None::<&str>)?, &MenuItem::with_id(app, "quit", "Выйти", true, None::<&str>)?])?;
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Polychrome")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id.as_ref() {
            "show" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                show_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn show_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkeys_parse() {
        for (keys, _) in HOTKEYS {
            keys.parse::<Shortcut>().unwrap();
        }
    }
}
