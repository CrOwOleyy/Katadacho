use std::path::Path;
use std::process::Command;

use tauri::{AppHandle, State};
use windows::core::{w, HSTRING};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use super::model::LauncherResult;
use super::{icons, reindex, Launcher};
use crate::controls::audio::ComGuard;
use crate::state::AppPaths;

/// ShellExecuteW renvoie une valeur supérieure à 32 en cas de succès.
const SHELL_EXECUTE_SUCCESS_THRESHOLD: isize = 32;

#[tauri::command]
pub fn search_launcher(launcher: State<'_, Launcher>, query: String) -> Vec<LauncherResult> {
    launcher.search(&query)
}

/// Lance une application, un fichier ou un dossier. Avec `run_as_admin`, Windows affiche
/// sa propre invite UAC.
#[tauri::command]
pub fn launch_item(path: String, run_as_admin: Option<bool>) -> Result<(), String> {
    let target = HSTRING::from(path.as_str());
    let result = if run_as_admin.unwrap_or(false) {
        unsafe { ShellExecuteW(None, w!("runas"), &target, None, None, SW_SHOWNORMAL) }
    } else {
        unsafe { ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL) }
    };

    if result.0 as isize > SHELL_EXECUTE_SUCCESS_THRESHOLD {
        Ok(())
    } else {
        Err(format!("Impossible de lancer « {path} » (code {})", result.0 as isize))
    }
}

/// Ouvre l'Explorateur avec l'élément sélectionné. Pour une application UWP, qui n'a pas de
/// chemin de fichier, on n'ouvre rien.
#[tauri::command]
pub fn open_item_location(path: String) -> Result<(), String> {
    if path.starts_with("shell:") {
        return Err("Cet élément n'a pas d'emplacement sur le disque".to_string());
    }
    if !Path::new(&path).exists() {
        return Err(format!("« {path} » n'existe plus"));
    }
    Command::new("explorer.exe")
        .arg(format!("/select,{path}"))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reindex_launcher(app: AppHandle) {
    reindex(app);
}

/// Icône PNG en base64 d'un élément, extraite à la demande et mise en cache sur le disque.
#[tauri::command]
pub async fn get_launcher_icon(paths: State<'_, AppPaths>, path: String) -> Result<String, String> {
    let cache_dir = paths.icon_cache();
    tauri::async_runtime::spawn_blocking(move || {
        let _com = ComGuard::new().map_err(|e| e.to_string())?;
        icons::icon_base64(&path, &cache_dir)
    })
    .await
    .map_err(|e| e.to_string())?
}
