pub mod machine;

use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager, State};
use windows::core::w;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC};

use crate::settings::{Settings, SettingsState, TimerSettings};
use machine::{Durations, Finished, Machine, Snapshot, Status};

pub const EVENT_TIMER_UPDATE: &str = "timer-update";
pub const EVENT_TIMER_FINISHED: &str = "timer-finished";
const SECONDS_PER_MINUTE: u32 = 60;
/// Marge ajoutée à l'attente pour se réveiller juste après le changement de seconde.
const WAKE_MARGIN: Duration = Duration::from_millis(5);

pub struct TimerState {
    machine: Mutex<Machine>,
    /// Réveille le thread quand une commande lance ou reprend le minuteur.
    wake: Condvar,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            machine: Mutex::new(Machine::new()),
            wake: Condvar::new(),
        }
    }
}

fn durations_from(timer: &TimerSettings) -> Durations {
    Durations {
        focus: timer.focus_minutes * SECONDS_PER_MINUTE,
        short_break: timer.short_break_minutes * SECONDS_PER_MINUTE,
        long_break: timer.long_break_minutes * SECONDS_PER_MINUTE,
        sessions_before_long_break: timer.sessions_before_long_break,
    }
}

fn play_finish_sound() {
    unsafe {
        let _ = PlaySoundW(w!("SystemNotification"), None, SND_ALIAS | SND_ASYNC);
    }
}

fn announce_finished(app: &AppHandle, finished: Finished, settings: &Settings) {
    if settings.timer.sound_enabled {
        play_finish_sound();
    }
    let _ = app.emit(EVENT_TIMER_FINISHED, finished);
}

/// Applique une action à la machine, puis réveille le thread et prévient le frontend.
fn apply<R>(
    app: &AppHandle,
    action: impl FnOnce(&mut Machine, Instant, &Durations, bool) -> R,
) -> (R, Snapshot, Settings) {
    let state = app.state::<TimerState>();
    let settings = app.state::<SettingsState>().snapshot();
    let durations = durations_from(&settings.timer);

    let now = Instant::now();
    let (result, snapshot) = {
        let mut machine = state.machine.lock().unwrap_or_else(|e| e.into_inner());
        let result = action(&mut machine, now, &durations, settings.timer.auto_start_next);
        (result, machine.snapshot(now))
    };

    state.wake.notify_all();
    let _ = app.emit(EVENT_TIMER_UPDATE, snapshot.clone());
    (result, snapshot, settings)
}

/// Thread de fond : dort tant que rien ne tourne, puis se réveille à chaque seconde pendant un décompte.
pub fn start(app: AppHandle) {
    thread::spawn(move || loop {
        let state = app.state::<TimerState>();
        let settings = app.state::<SettingsState>().snapshot();
        let durations = durations_from(&settings.timer);

        let (snapshot, finished, wait) = {
            let mut machine = state.machine.lock().unwrap_or_else(|e| e.into_inner());
            while machine.status() != Status::Running {
                machine = state.wake.wait(machine).unwrap_or_else(|e| e.into_inner());
            }

            let now = Instant::now();
            let finished = machine.tick(now, &durations, settings.timer.auto_start_next);
            let sub_second = machine.remaining(now).subsec_nanos();
            let wait = if sub_second == 0 {
                Duration::from_secs(1)
            } else {
                Duration::from_nanos(u64::from(sub_second))
            } + WAKE_MARGIN;
            (machine.snapshot(now), finished, wait)
        };

        let _ = app.emit(EVENT_TIMER_UPDATE, snapshot);
        if let Some(finished) = finished {
            announce_finished(&app, finished, &settings);
        }

        // Attente interrompue par une commande (pause, reset) ou par le changement de seconde.
        let machine = state.machine.lock().unwrap_or_else(|e| e.into_inner());
        let _ = state.wake.wait_timeout(machine, wait);
    });
}

#[tauri::command]
pub fn timer_get(state: State<'_, TimerState>) -> Snapshot {
    let machine = state.machine.lock().unwrap_or_else(|e| e.into_inner());
    machine.snapshot(Instant::now())
}

#[tauri::command]
pub fn timer_start_pomodoro(app: AppHandle) -> Snapshot {
    apply(&app, |machine, now, durations, _| machine.start_pomodoro(now, durations)).1
}

#[tauri::command]
pub fn timer_start_countdown(app: AppHandle, seconds: u32) -> Snapshot {
    apply(&app, |machine, now, _, _| machine.start_countdown(now, seconds)).1
}

#[tauri::command]
pub fn timer_pause(app: AppHandle) -> Snapshot {
    apply(&app, |machine, now, _, _| machine.pause(now)).1
}

#[tauri::command]
pub fn timer_resume(app: AppHandle) -> Snapshot {
    apply(&app, |machine, now, _, _| machine.resume(now)).1
}

#[tauri::command]
pub fn timer_reset(app: AppHandle) -> Snapshot {
    apply(&app, |machine, _, _, _| machine.reset()).1
}

/// Passe à la phase suivante du Pomodoro sans attendre la fin de la phase en cours.
#[tauri::command]
pub fn timer_skip(app: AppHandle) -> Snapshot {
    let (finished, snapshot, settings) =
        apply(&app, |machine, now, durations, auto| machine.skip(now, durations, auto));
    if let Some(finished) = finished {
        announce_finished(&app, finished, &settings);
    }
    snapshot
}
