mod idle;
mod live;
mod protocol;
mod zet;

use protocol::{Controller, MODE_OFF, MODE_STATIC, ZONES, Zone};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{
    AppHandle, Emitter, Manager, RunEvent, State, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Глобальные клавиши → событие `hotkey` в интерфейс (работают, пока программа запущена)
const HOTKEYS: [(&str, &str); 2] = [("Ctrl+Alt+L", "toggle"), ("Ctrl+Alt+P", "next")];

#[derive(Default)]
struct Hw {
    ctrl: Option<Controller>,
    zones: Vec<(usize, Zone)>, // что выбрал пользователь, даже пока подсветка погашена
    dirty: bool,
    dark: u8, // причины из idle: пока не 0, на железе всё выключено
    live: Option<live::Source>, // пока задан, на железе кадры live, а не зоны пользователя
    kbd: Option<Kbd>,
}

/// Клавиатура ZET через службу Blade Pro Software; необязательна — при ошибке связи просто отключаем
struct Kbd {
    link: zet::Zet,
    effect: zet::Effect, // что выбрал пользователь
    brights_max: u8,     // яркость статичного режима для живых кадров
}

type Shared = Mutex<Hw>;

#[derive(Serialize)]
struct ZoneView {
    id: usize,
    name: &'static str,
    mode: u8,
    rgb: [u8; 3],
    speed: u8,
    palette: Vec<[u8; 3]>,
}

impl Hw {
    fn lost(&mut self, e: hidapi::HidError) -> String {
        self.ctrl = None;
        format!("Связь с контроллером потеряна: {e}")
    }

    /// Вывести на железо то, что должно гореть сейчас: зоны пользователя или темноту
    fn show(&mut self) -> Result<(), String> {
        fn paint(c: &Controller, zones: &[(usize, Zone)], dark: bool) -> hidapi::HidResult<()> {
            zones.iter().try_for_each(|(id, z)| {
                let mut z = z.clone();
                if dark {
                    z.mode = MODE_OFF;
                }
                c.write_zone(*id, &z)
            })
        }
        let Some(c) = &self.ctrl else { return Err("Нет связи с контроллером".into()) };
        if paint(c, &self.zones, self.dark != 0).is_ok() {
            return Ok(());
        }
        // после сна старый хэндл бывает мёртв — пробуем открыть заново
        let c = Controller::open().map_err(|e| self.lost(e))?;
        paint(&c, &self.zones, self.dark != 0).map_err(|e| self.lost(e))?;
        self.ctrl = Some(c);
        Ok(())
    }

    /// Эффект пользователя на клавиатуру или темноту
    fn show_kbd(&mut self) {
        let Some(k) = &mut self.kbd else { return };
        let e = if self.dark != 0 { zet::Effect { mode: zet::MODE_STATIC, palette: vec![0], ..k.effect.clone() } } else { k.effect.clone() };
        if k.link.set_effect(&e).is_err() {
            self.kbd = None;
        }
    }

    fn set_dark(&mut self, reasons: u8) {
        let changed = (self.dark != 0) != (reasons != 0);
        self.dark = reasons;
        if changed && self.ctrl.is_some() {
            let _ = self.show(); // ошибка уже сбросила ctrl, интерфейс увидит её при следующей записи
        }
        if changed && self.live.is_none() {
            self.show_kbd();
        }
    }

    /// Кадр живого эффекта на клавиатуру — статичным цветом
    fn paint_kbd(&mut self, src: live::Source, rgb: [u8; 3]) {
        if self.live != Some(src) || self.dark != 0 {
            return;
        }
        let Some(k) = &mut self.kbd else { return };
        let e = zet::Effect { mode: zet::MODE_STATIC, brightness: k.brights_max, speed: 0, direction: 0, palette: vec![u32::from_be_bytes([0, rgb[0], rgb[1], rgb[2]])] };
        if k.link.set_effect(&e).is_err() {
            self.kbd = None;
        }
    }

    /// Свой цвет клавиатуры для «Музыки»; радуга и чёрный — белым
    fn kbd_base(&self) -> Option<[u8; 3]> {
        let c = self.kbd.as_ref()?.effect.palette.first().copied().unwrap_or(0);
        let [flags, r, g, b] = c.to_be_bytes();
        Some(if flags != 0 || c == 0 { [255; 3] } else { [r, g, b] })
    }

    /// Кадр живого эффекта: зоны, которые пользователь не выключил, — статичным цветом
    fn paint_live(&mut self, src: live::Source, color: impl Fn(&Zone) -> [u8; 3]) {
        if self.live != Some(src) || self.dark != 0 {
            return; // эффект выключили, пока считали кадр
        }
        let Some(c) = &self.ctrl else { return };
        let failed = self.zones.iter().filter(|(_, z)| z.mode != MODE_OFF).any(|(id, z)| {
            let mut f = z.clone();
            f.mode = MODE_STATIC;
            f.rgb = color(z);
            c.write_zone(*id, &f).is_err()
        });
        if failed {
            self.ctrl = None; // интерфейс увидит ошибку при следующей записи
        }
    }

    fn save(&mut self) -> Result<(), String> {
        let (dark, live) = (self.dark, self.live.take());
        // в память контроллера — зоны пользователя, а не погашенный вид или кадр эффекта
        self.dark = 0;
        let res = if dark != 0 || live.is_some() { self.show() } else { Ok(()) };
        let res = res.and_then(|()| {
            let Some(ctrl) = &self.ctrl else { return Err("Нет связи с контроллером".into()) };
            ctrl.save().map_err(|e| self.lost(e))
        });
        if res.is_ok() {
            self.dirty = false;
        }
        self.live = live;
        self.set_dark(dark);
        res
    }
}

#[tauri::command(async)]
fn connect(hw: State<'_, Shared>) -> Result<Vec<ZoneView>, String> {
    let mut hw = hw.lock().unwrap();
    hw.ctrl = None; // закрыть старый хэндл до повторного открытия
    let c = Controller::open().map_err(|e| format!("Контроллер ASRock не найден: {e}"))?;
    let zones = c
        .active_zones()
        .and_then(|ids| ids.into_iter().map(|z| c.read_zone(z).map(|zone| (z, zone))).collect::<hidapi::HidResult<Vec<_>>>())
        .map_err(|e| format!("Не удалось прочитать зоны: {e}"))?;
    let view = zones
        .iter()
        .map(|(id, z)| ZoneView { id: *id, name: ZONES[*id], mode: z.mode, rgb: z.rgb, speed: z.speed(), palette: z.palette.clone() })
        .collect();
    (hw.ctrl, hw.zones, hw.dirty) = (Some(c), zones, false);
    Ok(view)
}

#[tauri::command(async)]
fn write_zone(hw: State<'_, Shared>, id: usize, mode: u8, rgb: [u8; 3], speed: u8, palette: Vec<[u8; 3]>) -> Result<(), String> {
    let mut hw = hw.lock().unwrap();
    let Some((_, z)) = hw.zones.iter_mut().find(|(z, _)| *z == id) else { return Err(format!("Нет зоны {id}")) };
    z.set(mode, rgb, speed, palette);
    let z = z.clone();
    if hw.dark != 0 || hw.live.is_some() {
        hw.dirty = true; // на железе сейчас темнота или кадры эффекта; покажем, когда они закончатся
        return Ok(());
    }
    let Some(ctrl) = &hw.ctrl else { return Err("Нет связи с контроллером".into()) };
    match ctrl.write_zone(id, &z) {
        Ok(()) => {
            hw.dirty = true;
            Ok(())
        }
        Err(e) => Err(hw.lost(e)),
    }
}

#[tauri::command(async)]
fn save(hw: State<'_, Shared>) -> Result<(), String> {
    hw.lock().unwrap().save()
}

#[tauri::command(async)]
fn set_live(hw: State<'_, Shared>, source: Option<live::Source>) -> Result<(), String> {
    let mut hw = hw.lock().unwrap();
    hw.live = source;
    if source.is_none() && hw.dark == 0 {
        hw.show_kbd();
        if hw.ctrl.is_some() {
            hw.show()?; // вернуть зоны пользователя
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct KbdView {
    modes: Vec<zet::ModeInfo>,
    effect: zet::Effect,
}

/// Подключиться к службе ZET. mode — какой режим считать включённым: служба текущий не сообщает
#[tauri::command(async)]
fn kbd_open(hw: State<'_, Shared>, mode: u8) -> Result<KbdView, String> {
    let mut link = zet::Zet::connect().map_err(|e| format!("Служба ZET недоступна: {e}"))?;
    let modes = link.modes().map_err(|e| e.to_string())?;
    let effect = link.effect(mode).map_err(|e| e.to_string())?;
    let brights_max = modes.iter().find(|m| m.index == zet::MODE_STATIC).map_or(6, |m| m.brights.saturating_sub(1));
    hw.lock().unwrap().kbd = Some(Kbd { link, effect: effect.clone(), brights_max });
    Ok(KbdView { modes, effect })
}

/// Настройки режима, сохранённые в клавиатуре, — чтобы при смене режима показать его ползунки
#[tauri::command(async)]
fn kbd_effect(hw: State<'_, Shared>, mode: u8) -> Result<zet::Effect, String> {
    let mut guard = hw.lock().unwrap();
    let hw = &mut *guard;
    let Some(k) = &mut hw.kbd else { return Err("Клавиатура не подключена".into()) };
    k.link.effect(mode).map_err(|e| {
        hw.kbd = None;
        format!("Связь со службой ZET потеряна: {e}")
    })
}

#[tauri::command(async)]
fn kbd_set(hw: State<'_, Shared>, effect: zet::Effect) -> Result<(), String> {
    let mut guard = hw.lock().unwrap();
    let hw = &mut *guard;
    let Some(k) = &mut hw.kbd else { return Err("Клавиатура не подключена".into()) };
    k.effect = effect;
    if hw.dark == 0 && hw.live.is_none() {
        hw.show_kbd();
    }
    hw.kbd.as_ref().map(|_| ()).ok_or_else(|| "Связь со службой ZET потеряна".into())
}

#[tauri::command]
fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
fn set_autostart(app: AppHandle, on: bool) -> Result<(), String> {
    let a = app.autolaunch();
    if on { a.enable() } else { a.disable() }.map_err(|e| e.to_string())
}

fn show_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Плата, процессор, видеокарты и память из WMI — для устройств, которыми пока не управляем
#[tauri::command(async)]
fn system_info() -> Result<serde_json::Value, String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const SCRIPT: &str = r#"
[Console]::OutputEncoding = [Text.Encoding]::UTF8
@{
  board = Get-CimInstance Win32_BaseBoard | Select-Object Manufacturer, Product
  cpu = @(Get-CimInstance Win32_Processor | Select-Object Name)
  gpu = @(Get-CimInstance Win32_VideoController | Where-Object PNPDeviceID -like 'PCI*' | Select-Object Name, DriverVersion)
  ram = @(Get-CimInstance Win32_PhysicalMemory | Select-Object Manufacturer, PartNumber, Capacity, ConfiguredClockSpeed, DeviceLocator)
} | ConvertTo-Json -Depth 3 -Compress"#;
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;
    serde_json::from_slice(&out.stdout).map_err(|e| format!("WMI: {e}"))
}

/// Профили и имена зон — config.json рядом с exe: программа переносится вместе с настройками
fn config_path() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    exe.parent().map_or_else(|| PathBuf::from("config.json"), |dir| dir.join("config.json"))
}

/// None — файла ещё нет (первый запуск)
#[tauri::command(async)]
fn load_config() -> Result<Option<Value>, String> {
    let path = config_path();
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

#[tauri::command(async)]
fn save_config(config: Value) -> Result<(), String> {
    let path = config_path();
    let tmp = path.with_extension("json.tmp"); // через временный файл: обрыв записи не портит старый
    fs::create_dir_all(path.parent().unwrap())
        .and_then(|_| fs::write(&tmp, serde_json::to_vec_pretty(&config).unwrap()))
        .and_then(|_| fs::rename(&tmp, &path))
        .map_err(|e| format!("Не удалось сохранить настройки: {e}"))
}

#[tauri::command]
fn open_config_dir() -> Result<(), String> {
    let dir = config_path().parent().unwrap().to_owned();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::process::Command::new("explorer").arg(dir).spawn().map(|_| ()).map_err(|e| e.to_string())
}

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

/// `Polychrome.exe --profile Имя`: применить профиль, записать в контроллер и выйти — для Планировщика задач
fn apply_profile(name: &str) -> Result<(), String> {
    let config = load_config()?.ok_or("Нет файла настроек — запустите программу один раз")?;
    let profile = config["profiles"].as_array().into_iter().flatten().find(|p| p["name"] == name).ok_or_else(|| format!("Нет профиля «{name}»"))?;
    let hid = |e: hidapi::HidError| e.to_string();
    let c = Controller::open().map_err(|e| format!("Контроллер ASRock не найден: {e}"))?;
    for id in c.active_zones().map_err(hid)? {
        // ключ — номер зоны или '*' для всех, как profileLook() в app.svelte.ts
        let zones = &profile["zones"];
        let l = match &zones[id.to_string()] {
            Value::Null => &zones["*"],
            l => l,
        };
        if l.is_null() {
            continue;
        }
        let l = Look::deserialize(l).map_err(|e| format!("Профиль «{name}», зона {id}: {e}"))?;
        let mut z = c.read_zone(id).map_err(hid)?;
        z.set(l.mode, l.dim(l.color), l.speed, l.palette.iter().map(|&p| l.dim(p)).collect());
        c.write_zone(id, &z).map_err(hid)?;
    }
    c.save().map_err(hid)?;
    if let Some(k) = profile.get("kbd").filter(|k| !k.is_null()) {
        let e = zet::Effect::deserialize(k).map_err(|e| format!("Профиль «{name}», клавиатура: {e}"))?;
        zet::Zet::connect().and_then(|mut z| z.set_effect(&e)).map_err(|e| format!("Клавиатура ZET: {e}"))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let [flag, name] = &args[..]
        && flag == "--profile"
    {
        let code = apply_profile(name).map_or_else(|e| { eprintln!("{e}"); 1 }, |()| 0);
        std::process::exit(code);
    }

    tauri::Builder::default()
        // первым: второй запуск exe только показывает окно уже работающей копии
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show_window(app)))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--tray"])))
        .manage(Shared::default())
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
        .setup(|app| {
            for (keys, _) in HOTKEYS {
                // сочетание может быть занято другой программой — работаем без него
                if let Err(e) = app.global_shortcut().register(keys) {
                    eprintln!("{keys}: {e}");
                }
            }

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

            // автозапуск передаёт --tray: стартуем без окна (в tauri.conf.json окно скрыто)
            if !std::env::args().any(|a| a == "--tray") {
                show_window(app.handle());
            }

            live::run(app.handle().clone());
            let handle = app.handle().clone();
            idle::watch(move |why, started| {
                let mut hw = handle.state::<Shared>().inner().lock().unwrap();
                let reasons = if started { hw.dark | why } else { hw.dark & !why };
                hw.set_dark(reasons);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![connect, write_zone, save, system_info, load_config, save_config, open_config_dir, get_autostart, set_autostart, set_live, kbd_open, kbd_effect, kbd_set])
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
                let mut hw = app.state::<Shared>().inner().lock().unwrap();
                if hw.dirty {
                    let _ = hw.save(); // не потерять настройку при выходе
                }
            }
        });
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

    #[test]
    fn hotkeys_parse() {
        for (keys, _) in HOTKEYS {
            keys.parse::<Shortcut>().unwrap();
        }
    }
}
