//! Volume par application sous Windows : chaque application qui joue du son ouvre une session audio
//! sur le périphérique de sortie, avec son propre volume (`ISimpleAudioVolume`).

use windows::core::{Interface, Result as WinResult, PWSTR};
use windows::Win32::Foundation::{CloseHandle, S_OK};
use windows::Win32::Media::Audio::{
    eConsole, eRender, AudioSessionStateExpired, IAudioSessionControl, IAudioSessionControl2,
    IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

use super::audio::take_pwstr;
use super::mixer::{display_name, Stream, SYSTEM_SOUNDS_KEY};

/// Une session vivante, avec de quoi la régler.
struct Session {
    stream: Stream,
    volume: ISimpleAudioVolume,
}

/// Chemin de l'exécutable d'un processus. Échoue pour les processus protégés.
fn executable_path(pid: u32) -> Option<String> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut buffer = [0u16; 1024];
    let mut size = buffer.len() as u32;
    let queried =
        unsafe { QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut size) };
    let _ = unsafe { CloseHandle(process) };
    queried.ok()?;
    Some(String::from_utf16_lossy(&buffer[..size as usize]))
}

/// Nom de l'application inscrit dans l'exécutable (« Description du fichier » dans ses propriétés),
/// par exemple « Steam Client WebHelper » pour `steamwebhelper.exe`.
fn file_description(path: &str) -> Option<String> {
    use std::ffi::c_void;
    use windows::core::{w, HSTRING};
    use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};

    let file = HSTRING::from(path);
    let size = unsafe { GetFileVersionInfoSizeW(&file, None) };
    if size == 0 {
        return None;
    }
    let mut data = vec![0u8; size as usize];
    unsafe { GetFileVersionInfoW(&file, None, size, data.as_mut_ptr().cast()) }.ok()?;

    // La description est rangée sous la langue et la page de code déclarées par le fichier.
    let mut pointer: *mut c_void = std::ptr::null_mut();
    let mut length = 0u32;
    let found = unsafe { VerQueryValueW(data.as_ptr().cast(), w!("\\VarFileInfo\\Translation"), &mut pointer, &mut length) };
    if !found.as_bool() || length < 4 {
        return None;
    }
    let (language, code_page) = unsafe { (*(pointer as *const u16), *(pointer as *const u16).add(1)) };

    let key = HSTRING::from(format!("\\StringFileInfo\\{language:04x}{code_page:04x}\\FileDescription"));
    let found = unsafe { VerQueryValueW(data.as_ptr().cast(), &key, &mut pointer, &mut length) };
    if !found.as_bool() || length == 0 {
        return None;
    }
    let text = unsafe { std::slice::from_raw_parts(pointer as *const u16, length as usize) };
    let description = String::from_utf16_lossy(text).trim_end_matches('\0').trim().to_string();
    (!description.is_empty()).then_some(description)
}

fn describe(control: &IAudioSessionControl) -> WinResult<Option<Session>> {
    if unsafe { control.GetState()? } == AudioSessionStateExpired {
        return Ok(None);
    }
    let control2: IAudioSessionControl2 = control.cast()?;
    let volume: ISimpleAudioVolume = control.cast()?;
    let level = (unsafe { volume.GetMasterVolume()? } * 100.0).round().clamp(0.0, 100.0) as u8;
    let muted = unsafe { volume.GetMute()? }.as_bool();

    let (key, name) = if unsafe { control2.IsSystemSoundsSession() } == S_OK {
        (SYSTEM_SOUNDS_KEY.to_string(), "Sons système".to_string())
    } else {
        let pid = unsafe { control2.GetProcessId()? };
        match executable_path(pid) {
            Some(path) => {
                let executable = path.rsplit('\\').next().unwrap_or(&path).to_lowercase();
                (executable, file_description(&path).unwrap_or_else(|| display_name(&path)))
            }
            // Processus inaccessible : on garde la session, identifiée par son instance.
            None => {
                let instance = take_pwstr(unsafe { control2.GetSessionInstanceIdentifier()? })?;
                let label = take_pwstr(unsafe { control.GetDisplayName()? }).unwrap_or_default();
                let name = if label.is_empty() || label.starts_with('@') { "Application".to_string() } else { label };
                (instance, name)
            }
        }
    };
    Ok(Some(Session { stream: Stream { key, name, volume: level, muted }, volume }))
}

/// Sessions du périphérique de sortie par défaut. Le COM doit être initialisé.
fn sessions() -> WinResult<Vec<Session>> {
    let enumerator: IMMDeviceEnumerator = unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole)? };
    let manager: IAudioSessionManager2 = unsafe { device.Activate(CLSCTX_ALL, None)? };
    let list = unsafe { manager.GetSessionEnumerator()? };

    let mut sessions = Vec::new();
    for index in 0..unsafe { list.GetCount()? } {
        // Une session qui disparaît pendant la lecture est simplement ignorée.
        if let Ok(Some(session)) = unsafe { list.GetSession(index) }.and_then(|control| describe(&control)) {
            sessions.push(session);
        }
    }
    Ok(sessions)
}

pub fn streams() -> WinResult<Vec<Stream>> {
    Ok(sessions()?.into_iter().map(|session| session.stream).collect())
}

fn matching(id: &str) -> Result<Vec<Session>, String> {
    let found: Vec<Session> =
        sessions().map_err(|e| e.to_string())?.into_iter().filter(|session| session.stream.key == id).collect();
    if found.is_empty() {
        Err("Cette application ne joue plus de son".to_string())
    } else {
        Ok(found)
    }
}

pub fn set_volume(id: &str, level: u8) -> Result<(), String> {
    let scalar = f32::from(level.min(100)) / 100.0;
    for session in matching(id)? {
        unsafe { session.volume.SetMasterVolume(scalar, std::ptr::null()) }.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Inverse le mute de toutes les sessions de l'application et retourne le nouvel état.
pub fn toggle_mute(id: &str) -> Result<bool, String> {
    let found = matching(id)?;
    let next = !found.iter().all(|session| session.stream.muted);
    for session in found {
        unsafe { session.volume.SetMute(next, std::ptr::null()) }.map_err(|e| e.to_string())?;
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::ComGuard;
    use crate::controls::mixer::group;

    /// Test manuel sur la machine : `cargo test --lib lists_real_sessions -- --ignored --nocapture`.
    #[test]
    #[ignore = "dépend des applications qui jouent du son"]
    fn lists_real_sessions() {
        let _com = ComGuard::new().expect("COM");
        let apps = group(streams().expect("sessions audio"));
        for app in &apps {
            println!("{} | {} | {} % | muet : {}", app.id, app.name, app.volume, app.muted);
        }
        assert!(!apps.is_empty(), "au moins les sons système existent");
        // Réapplique le volume actuel : exerce l'écriture sans rien changer pour l'utilisateur.
        for app in &apps {
            set_volume(&app.id, app.volume).expect("réglage du volume");
        }
        let after = group(streams().expect("sessions audio"));
        assert_eq!(after.iter().map(|app| app.volume).collect::<Vec<_>>(), apps.iter().map(|app| app.volume).collect::<Vec<_>>());
    }
}
