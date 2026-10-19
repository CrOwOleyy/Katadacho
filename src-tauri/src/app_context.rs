//! Application au premier plan : nom de son exécutable et titre de sa fenêtre. Le frontend s'en
//! sert pour reconnaître une vidéo en cours (lecteur, site de streaming).

use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ForegroundApp {
    /// Nom de l'exécutable, par exemple `vlc.exe`.
    pub process: String,
    /// Titre de la fenêtre, par exemple « Épisode 3 - Crunchyroll - Google Chrome ».
    pub title: String,
}

/// Dernier segment d'un chemin Windows ou Unix : `C:\Program Files\vlc.exe` donne `vlc.exe`.
fn file_name_of(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

#[cfg(windows)]
fn foreground_app() -> Option<ForegroundApp> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};

    let window = unsafe { GetForegroundWindow() };
    if window.0.is_null() {
        return None;
    }

    let mut title_buffer = [0u16; 512];
    let length = (unsafe { GetWindowTextW(window, &mut title_buffer) }.max(0) as usize).min(title_buffer.len());
    let title = String::from_utf16_lossy(&title_buffer[..length]);

    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
    // Interroger un processus protégé peut échouer : dans ce cas on ne sait pas, donc aucune application.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut path = [0u16; 1024];
    let mut size = path.len() as u32;
    let queried = unsafe {
        QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(path.as_mut_ptr()), &mut size)
    };
    let _ = unsafe { CloseHandle(process) };
    queried.ok()?;

    let full_path = String::from_utf16_lossy(&path[..size as usize]);
    Some(ForegroundApp { process: file_name_of(&full_path).to_string(), title })
}

/// Pas encore d'équivalent sous Linux : le suivi des vidéos reste inactif.
#[cfg(not(windows))]
fn foreground_app() -> Option<ForegroundApp> {
    None
}

#[tauri::command]
pub fn get_foreground_app() -> Option<ForegroundApp> {
    foreground_app()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_executable_name_is_taken_from_a_windows_or_unix_path() {
        assert_eq!(file_name_of(r"C:\Program Files\VideoLAN\VLC\vlc.exe"), "vlc.exe");
        assert_eq!(file_name_of("/usr/bin/mpv"), "mpv");
        assert_eq!(file_name_of("seul.exe"), "seul.exe");
    }
}
