//! События Windows, на время которых подсветку надо погасить: блокировка, сон, выключенный монитор.

use std::{
    ptr::{null, null_mut},
    sync::OnceLock,
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::{
        LibraryLoader::GetModuleHandleW,
        Power::{POWERBROADCAST_SETTING, RegisterPowerSettingNotification},
        RemoteDesktop::{NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification},
        SystemServices::GUID_CONSOLE_DISPLAY_STATE,
    },
    UI::WindowsAndMessaging::*,
};

/// Причины погасить подсветку — биты, потому что могут наложиться (заблокировал, потом погас монитор)
pub const LOCKED: u8 = 1;
pub const ASLEEP: u8 = 2;
pub const DISPLAY_OFF: u8 = 4;

static HANDLER: OnceLock<Box<dyn Fn(u8, bool) + Send + Sync>> = OnceLock::new();

/// `f(причина, началась ли)` вызывается из отдельного потока со своим скрытым окном
pub fn watch(f: impl Fn(u8, bool) + Send + Sync + 'static) {
    if HANDLER.set(Box::new(f)).is_err() {
        return;
    }
    std::thread::spawn(|| unsafe {
        let class = windows_sys::core::w!("PolychromeIdle");
        let wc = WNDCLASSW { lpfnWndProc: Some(wndproc), hInstance: GetModuleHandleW(null()), lpszClassName: class, ..Default::default() };
        RegisterClassW(&wc);
        // обычное невидимое окно, а не message-only: тем не приходит широковещательный WM_POWERBROADCAST
        let hwnd = CreateWindowExW(0, class, class, 0, 0, 0, 0, 0, null_mut(), null_mut(), wc.hInstance, null());
        WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);
        RegisterPowerSettingNotification(hwnd, &GUID_CONSOLE_DISPLAY_STATE, DEVICE_NOTIFY_WINDOW_HANDLE);
        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            DispatchMessageW(&msg);
        }
    });
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let f = HANDLER.get().unwrap();
    match (msg, wp as u32) {
        (WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK) => f(LOCKED, true),
        (WM_WTSSESSION_CHANGE, WTS_SESSION_UNLOCK) => f(LOCKED, false),
        // после возврата из обработчика система засыпает, поэтому гасим синхронно
        (WM_POWERBROADCAST, PBT_APMSUSPEND) => f(ASLEEP, true),
        (WM_POWERBROADCAST, PBT_APMRESUMEAUTOMATIC) => f(ASLEEP, false),
        (WM_POWERBROADCAST, PBT_POWERSETTINGCHANGE) => {
            // подписаны только на GUID_CONSOLE_DISPLAY_STATE: 0 — выключен, 1 — включён, 2 — притушен
            let s = unsafe { &*(lp as *const POWERBROADCAST_SETTING) };
            f(DISPLAY_OFF, s.Data[0] == 0);
        }
        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
}
