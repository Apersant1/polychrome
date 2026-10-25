mod protocol;

use protocol::{Controller, MAX_PALETTE, ZONES, Zone};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, State, WindowEvent};

#[derive(Default)]
struct Hw {
    ctrl: Option<Controller>,
    zones: Vec<(usize, Zone)>,
    dirty: bool,
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
    z.mode = mode;
    z.rgb = rgb;
    z.set_speed(speed.min(100));
    if (2..=MAX_PALETTE).contains(&palette.len()) {
        z.palette = palette;
    }
    let z = z.clone();
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
    let mut hw = hw.lock().unwrap();
    let Some(ctrl) = &hw.ctrl else { return Err("Нет связи с контроллером".into()) };
    match ctrl.save() {
        Ok(()) => {
            hw.dirty = false;
            Ok(())
        }
        Err(e) => Err(hw.lost(e)),
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Shared::default())
        .invoke_handler(tauri::generate_handler![connect, write_zone, save, system_info])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                let mut hw = window.state::<Shared>().inner().lock().unwrap();
                if hw.dirty && hw.ctrl.as_ref().is_some_and(|c| c.save().is_ok()) {
                    hw.dirty = false; // не потерять настройку при закрытии
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
