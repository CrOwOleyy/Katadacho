//! Mémorise la dernière fenêtre active qui n'appartient pas à l'application, pour pouvoir
//! lui renvoyer le focus et simuler `Ctrl+V` après avoir cliqué sur un élément de l'Island.

use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Once;
use std::thread;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId, TranslateMessage,
    EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT,
};

static LAST_EXTERNAL: AtomicIsize = AtomicIsize::new(0);
static STARTED: Once = Once::new();

fn remember(window: HWND) {
    if window.0.is_null() {
        return;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
    if pid != 0 && pid != unsafe { GetCurrentProcessId() } {
        LAST_EXTERNAL.store(window.0 as isize, Ordering::SeqCst);
    }
}

unsafe extern "system" fn on_foreground(
    _hook: HWINEVENTHOOK,
    _event: u32,
    window: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    remember(window);
}

/// Lance le suivi du premier plan dans un thread dédié. Sans effet si déjà démarré.
pub fn start() {
    STARTED.call_once(|| {
        thread::spawn(|| {
            remember(unsafe { GetForegroundWindow() });
            let hook = unsafe {
                SetWinEventHook(
                    EVENT_SYSTEM_FOREGROUND,
                    EVENT_SYSTEM_FOREGROUND,
                    None,
                    Some(on_foreground),
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT,
                )
            };
            if hook.0.is_null() {
                eprintln!("foreground tracking unavailable");
                return;
            }

            // Les hooks hors contexte exigent une boucle de messages sur le thread qui les installe.
            let mut message = MSG::default();
            while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
                unsafe {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        });
    });
}

/// Dernière fenêtre externe connue, si elle existe encore.
pub fn last_external_window() -> Option<HWND> {
    let raw = LAST_EXTERNAL.load(Ordering::SeqCst);
    (raw != 0).then(|| HWND(raw as *mut std::ffi::c_void))
}
