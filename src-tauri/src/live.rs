//! Живые эффекты поверх настроек пользователя: ~30 кадров/с по звуку или по экрану.
//! Кадры не меняют зоны пользователя и не попадают в память контроллера (см. Hw::paint_live).

use crate::Shared;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::Deserialize;
use std::{
    mem::{size_of, zeroed},
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering::Relaxed},
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
use windows_sys::Win32::{Graphics::Gdi::*, UI::WindowsAndMessaging::*};

#[derive(Clone, Copy, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Music,
    Screen,
}

pub fn run(app: AppHandle) {
    std::thread::spawn(move || {
        let mut audio = None;
        let mut env = Envelope::default();
        let mut screen = [0f32; 3];
        let mut frame = Instant::now();
        loop {
            // ~30 кадров/с с учётом времени на захват (экран — 10–25 мс)
            frame += Duration::from_millis(33);
            std::thread::sleep(frame.saturating_duration_since(Instant::now()));
            frame = frame.max(Instant::now() - Duration::from_millis(33)); // после долгой паузы не догоняем пропущенное
            let state = app.state::<Shared>();
            let src = {
                let hw = state.lock().unwrap();
                if hw.dark != 0 || hw.ctrl.is_none() { None } else { hw.live }
            };
            // захват звука держим открытым, только пока он нужен
            if src != Some(Source::Music) {
                audio = None;
            } else if audio.is_none() {
                audio = loopback().map_err(|e| eprintln!("звук: {e}")).ok();
            }
            match src {
                Some(Source::Music) => {
                    let rms = audio.as_ref().map_or(0.0, |(_, level): &(cpal::Stream, Arc<AtomicU32>)| f32::from_bits(level.swap(0, Relaxed)));
                    let k = env.next(rms);
                    state.lock().unwrap().paint_live(Source::Music, |z| scale(z.rgb, k));
                }
                Some(Source::Screen) => {
                    let Some(c) = screen_color() else { continue };
                    for i in 0..3 {
                        screen[i] += (c[i] as f32 - screen[i]) * 0.3; // сглаживание, чтобы не мерцало на сменах кадра
                    }
                    let c = screen.map(|v| v.round() as u8);
                    state.lock().unwrap().paint_live(Source::Screen, |_| c);
                }
                None => {}
            }
        }
    });
}

fn scale(c: [u8; 3], k: f32) -> [u8; 3] {
    c.map(|v| (v as f32 * k.clamp(0.0, 1.0)).round() as u8)
}

/// Громкость 0..1: мгновенная атака, плавный спад и автоусиление под текущую громкость системы
#[derive(Default)]
struct Envelope {
    level: f32,
    peak: f32,
}

impl Envelope {
    fn next(&mut self, rms: f32) -> f32 {
        self.level = rms.max(self.level * 0.85);
        self.peak = self.level.max(self.peak * 0.998).max(0.01); // тише 0.01 — почти тишина, не усиливаем
        self.level / self.peak
    }
}

/// Что играет в колонках (WASAPI loopback): поток пишет максимум RMS блоков с прошлого кадра
fn loopback() -> Result<(cpal::Stream, Arc<AtomicU32>), String> {
    let dev = cpal::default_host().default_output_device().ok_or("нет устройства вывода")?;
    let cfg = dev.default_output_config().map_err(|e| e.to_string())?;
    // shortcut: только f32 — формат WASAPI по умолчанию; для других понадобится конвертация
    if cfg.sample_format() != cpal::SampleFormat::F32 {
        return Err(format!("формат {:?} не поддерживается", cfg.sample_format()));
    }
    let level = Arc::new(AtomicU32::new(0));
    let l = level.clone();
    let stream = dev
        .build_input_stream::<f32, _, _>(
            cfg.config(),
            move |data, _| {
                let rms = (data.iter().map(|s| s * s).sum::<f32>() / data.len().max(1) as f32).sqrt();
                l.fetch_max(rms.to_bits(), Relaxed); // для неотрицательных f32 порядок битов совпадает с порядком чисел
            },
            |e| eprintln!("звук: {e}"),
            None,
        )
        .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok((stream, level))
}

/// Средний цвет основного монитора.
/// shortcut: GDI не видит эксклюзивный полноэкранный режим игр — там нужен DXGI Desktop Duplication.
fn screen_color() -> Option<[u8; 3]> {
    const W: i32 = 64;
    const H: i32 = 36;
    unsafe {
        let screen = GetDC(null_mut());
        let mem = CreateCompatibleDC(screen);
        let bmp = CreateCompatibleBitmap(screen, W, H);
        let old = SelectObject(mem, bmp);
        // COLORONCOLOR берёт отдельные пиксели без усреднения, зато быстро; шум гасит среднее по 64×36 точкам
        SetStretchBltMode(mem, COLORONCOLOR);
        let ok = StretchBlt(mem, 0, 0, W, H, screen, 0, 0, GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN), SRCCOPY) != 0;
        SelectObject(mem, old); // GetDIBits требует, чтобы битмап не был выбран в DC
        let mut bi: BITMAPINFO = zeroed();
        bi.bmiHeader = BITMAPINFOHEADER { biSize: size_of::<BITMAPINFOHEADER>() as u32, biWidth: W, biHeight: -H, biPlanes: 1, biBitCount: 32, ..zeroed() };
        let mut px = vec![0u8; (W * H * 4) as usize];
        let got = GetDIBits(mem, bmp, 0, H as u32, px.as_mut_ptr().cast(), &mut bi, DIB_RGB_COLORS);
        DeleteObject(bmp);
        DeleteDC(mem);
        ReleaseDC(null_mut(), screen);
        if !ok || got == 0 {
            return None;
        }
        let mut sum = [0u32; 3];
        for p in px.chunks_exact(4) {
            sum[0] += p[2] as u32; // BGRA
            sum[1] += p[1] as u32;
            sum[2] += p[0] as u32;
        }
        Some(sum.map(|s| (s / (W * H) as u32) as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_follows_loudness() {
        let mut e = Envelope::default();
        assert_eq!(e.next(0.0), 0.0);
        for _ in 0..30 {
            e.next(0.3);
        }
        assert!(e.next(0.3) > 0.99);
        for _ in 0..30 {
            e.next(0.03); // стало тише: спад, но автоусиление ещё помнит громкий пик
        }
        let k = e.next(0.03);
        assert!(k > 0.05 && k < 0.2, "{k}");
    }

    #[test]
    #[ignore] // нужен монитор: cargo test -- --ignored --nocapture screen_color_fast
    fn screen_color_fast() {
        let c = screen_color().unwrap();
        let t = Instant::now();
        for _ in 0..10 {
            screen_color().unwrap();
        }
        println!("{c:?}, в среднем {:?} на кадр", t.elapsed() / 10);
    }
}
