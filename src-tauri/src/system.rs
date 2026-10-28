//! Плата, процессор, видеокарты и память из WMI — для устройств, которыми пока не управляем.

use serde_json::Value;
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

pub fn info() -> Result<Value, String> {
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;
    serde_json::from_slice(&out.stdout).map_err(|e| format!("WMI: {e}"))
}
