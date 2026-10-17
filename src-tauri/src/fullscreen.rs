//! Détecte si l'application au premier plan occupe tout l'écran (jeu, vidéo, présentation).

#[cfg(windows)]
fn foreground_is_fullscreen() -> bool {
    use std::mem::size_of;

    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    use windows::Win32::System::Threading::GetCurrentProcessId;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId,
    };

    /// Le bureau et la barre des tâches couvrent l'écran sans être une application plein écran.
    const SHELL_CLASSES: &[&str] = &["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() {
        return false;
    }

    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
    if pid == unsafe { GetCurrentProcessId() } {
        return false;
    }

    let mut class = [0u16; 64];
    let length = unsafe { GetClassNameW(window, &mut class) } as usize;
    let class_name = String::from_utf16_lossy(&class[..length.min(class.len())]);
    if SHELL_CLASSES.contains(&class_name.as_str()) {
        return false;
    }

    let mut rect = RECT::default();
    if unsafe { GetWindowRect(window, &mut rect) }.is_err() {
        return false;
    }
    let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        return false;
    }

    // La fenêtre couvre tout l'écran, barre des tâches comprise.
    let screen = info.rcMonitor;
    rect.left <= screen.left && rect.top <= screen.top && rect.right >= screen.right && rect.bottom >= screen.bottom
}

#[cfg(not(windows))]
fn foreground_is_fullscreen() -> bool {
    false
}

#[tauri::command]
pub fn is_fullscreen_app_active() -> bool {
    foreground_is_fullscreen()
}
