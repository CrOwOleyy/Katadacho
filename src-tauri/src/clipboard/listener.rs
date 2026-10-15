//! Écoute événementielle du presse-papier : une fenêtre invisible reçoit `WM_CLIPBOARDUPDATE`
//! à chaque copie, sans aucun polling.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;
use std::thread;

use tauri::AppHandle;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{AddClipboardFormatListener, GetClipboardSequenceNumber};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW, TranslateMessage,
    HWND_MESSAGE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLIPBOARDUPDATE, WNDCLASSW,
};

static APP: OnceLock<AppHandle> = OnceLock::new();
static LAST_SEQUENCE: AtomicU32 = AtomicU32::new(0);

/// Lance le thread d'écoute. Doit être appelé une seule fois, au démarrage.
pub fn start(app: AppHandle) {
    if APP.set(app).is_err() {
        return;
    }
    thread::spawn(|| {
        if let Err(error) = unsafe { run() } {
            eprintln!("clipboard listener stopped: {error}");
        }
    });
}

unsafe fn run() -> windows::core::Result<()> {
    let instance = unsafe { GetModuleHandleW(None)? };
    let class_name = w!("KatadachoClipboardListener");

    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: instance.into(),
        lpszClassName: class_name,
        ..Default::default()
    };
    unsafe { RegisterClassW(&class) };

    let window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!(""),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(instance.into()),
            None,
        )?
    };
    unsafe { AddClipboardFormatListener(window)? };

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

unsafe extern "system" fn window_proc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_CLIPBOARDUPDATE {
        // Une même copie déclenche souvent plusieurs notifications : on ignore les répétitions.
        let sequence = unsafe { GetClipboardSequenceNumber() };
        if LAST_SEQUENCE.swap(sequence, Ordering::SeqCst) != sequence {
            if let Some(app) = APP.get() {
                super::capture(app, window);
            }
        }
        return LRESULT(0);
    }
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}
