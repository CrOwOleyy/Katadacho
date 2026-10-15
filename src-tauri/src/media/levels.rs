//! Niveau sonore en direct du périphérique de sortie par défaut (tout ce que tu entends),
//! lu via `IAudioMeterInformation`. Le thread ne tourne que lorsque le frontend le demande.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::controls::audio::ComGuard;

pub const EVENT_AUDIO_LEVEL: &str = "audio-level";
const TICK: Duration = Duration::from_millis(33);
/// Le périphérique par défaut peut changer (casque branché) : on le relit régulièrement.
const REACQUIRE_EVERY_TICKS: u32 = 90;

static RUNNING: AtomicBool = AtomicBool::new(false);

fn acquire_meter() -> Option<IAudioMeterInformation> {
    let enumerator: IMMDeviceEnumerator = unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }.ok()?;
    let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole) }.ok()?;
    unsafe { device.Activate::<IAudioMeterInformation>(CLSCTX_ALL, None) }.ok()
}

fn start(app: AppHandle) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(move || {
        let Ok(_com) = ComGuard::new() else {
            RUNNING.store(false, Ordering::SeqCst);
            return;
        };

        let mut meter = acquire_meter();
        let mut ticks = 0u32;

        while RUNNING.load(Ordering::SeqCst) {
            if ticks % REACQUIRE_EVERY_TICKS == 0 {
                meter = acquire_meter();
            }
            ticks = ticks.wrapping_add(1);

            let peak = meter
                .as_ref()
                .and_then(|meter| unsafe { meter.GetPeakValue() }.ok())
                .unwrap_or(0.0);

            if app.emit(EVENT_AUDIO_LEVEL, peak).is_err() {
                break;
            }
            thread::sleep(TICK);
        }
        RUNNING.store(false, Ordering::SeqCst);
    });
}

#[tauri::command]
pub fn start_audio_levels(app: AppHandle) {
    start(app);
}

#[tauri::command]
pub fn stop_audio_levels() {
    RUNNING.store(false, Ordering::SeqCst);
}
