use std::path::Path;

use tauri::{AppHandle, State};

use super::model::LauncherResult;
use super::{reindex, Launcher};
use crate::state::AppPaths;

#[tauri::command]
pub fn search_launcher(launcher: State<'_, Launcher>, query: String) -> Vec<LauncherResult> {
    launcher.search(&query)
}

/// Lance une application, un fichier ou un dossier. Avec `run_as_admin`, Windows affiche
/// sa propre invite UAC.
#[tauri::command]
pub fn launch_item(path: String, run_as_admin: Option<bool>) -> Result<(), String> {
    imp::launch(&path, run_as_admin.unwrap_or(false))
}

/// Ouvre le gestionnaire de fichiers sur l'élément. Pour une application du Microsoft Store,
/// qui n'a pas de chemin de fichier, on n'ouvre rien.
#[tauri::command]
pub fn open_item_location(path: String) -> Result<(), String> {
    if path.starts_with("shell:") {
        return Err("Cet élément n'a pas d'emplacement sur le disque".to_string());
    }
    if !Path::new(&path).exists() {
        return Err(format!("« {path} » n'existe plus"));
    }
    imp::reveal(&path)
}

#[tauri::command]
pub fn reindex_launcher(app: AppHandle) {
    reindex(app);
}

/// Icône PNG en base64 d'un élément, extraite à la demande et mise en cache sur le disque.
#[tauri::command]
pub async fn get_launcher_icon(paths: State<'_, AppPaths>, path: String) -> Result<String, String> {
    let cache_dir = paths.icon_cache();
    tauri::async_runtime::spawn_blocking(move || imp::icon(&path, &cache_dir))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(windows)]
mod imp {
    use std::path::Path;
    use std::process::Command;

    use windows::core::{w, HSTRING};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    use crate::com::ComGuard;
    use crate::launcher::icons;

    /// ShellExecuteW renvoie une valeur supérieure à 32 en cas de succès.
    const SHELL_EXECUTE_SUCCESS_THRESHOLD: isize = 32;

    pub fn launch(path: &str, as_admin: bool) -> Result<(), String> {
        let target = HSTRING::from(path);
        let verb = if as_admin { w!("runas") } else { w!("open") };
        let result = unsafe { ShellExecuteW(None, verb, &target, None, None, SW_SHOWNORMAL) };

        if result.0 as isize > SHELL_EXECUTE_SUCCESS_THRESHOLD {
            Ok(())
        } else {
            Err(format!("Impossible de lancer « {path} » (code {})", result.0 as isize))
        }
    }

    pub fn reveal(path: &str) -> Result<(), String> {
        Command::new("explorer.exe")
            .arg(format!("/select,{path}"))
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn icon(path: &str, cache_dir: &Path) -> Result<String, String> {
        let _com = ComGuard::new().map_err(|e| e.to_string())?;
        icons::icon_base64(path, cache_dir)
    }
}

/// Linux et autres systèmes freedesktop : `gio launch` pour les applications `.desktop`,
/// `xdg-open` pour le reste, comme le font les bureaux eux-mêmes.
#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use std::path::Path;
    use std::process::{Command, Stdio};

    use crate::platform::unsupported;

    fn spawn(program: &str, arguments: &[&str]) -> Result<(), String> {
        Command::new(program)
            .args(arguments)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Impossible de lancer {program} : {error}"))
    }

    pub fn launch(path: &str, as_admin: bool) -> Result<(), String> {
        if as_admin {
            return Err(unsupported("Le lancement en administrateur"));
        }
        if path.ends_with(".desktop") {
            spawn("gio", &["launch", path])
        } else {
            spawn("xdg-open", &[path])
        }
    }

    pub fn reveal(path: &str) -> Result<(), String> {
        let folder = Path::new(path)
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string());
        spawn("xdg-open", &[&folder])
    }

    /// Les icônes Linux sont déjà des fichiers PNG du thème d'icônes : elles sont lues telles quelles.
    pub fn icon(path: &str, _cache_dir: &Path) -> Result<String, String> {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;

        if !path.to_ascii_lowercase().ends_with(".png") {
            return Err(unsupported("Les icônes qui ne sont pas en PNG"));
        }
        std::fs::read(path).map(|bytes| STANDARD.encode(bytes)).map_err(|e| e.to_string())
    }
}

/// macOS : la commande `open` gère applications, fichiers et dossiers.
#[cfg(target_os = "macos")]
mod imp {
    use std::path::Path;
    use std::process::Command;

    use crate::platform::unsupported;

    pub fn launch(path: &str, as_admin: bool) -> Result<(), String> {
        if as_admin {
            return Err(unsupported("Le lancement en administrateur"));
        }
        Command::new("open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
    }

    pub fn reveal(path: &str) -> Result<(), String> {
        Command::new("open").args(["-R", path]).spawn().map(|_| ()).map_err(|e| e.to_string())
    }

    pub fn icon(_path: &str, _cache_dir: &Path) -> Result<String, String> {
        Err(unsupported("Les icônes d'applications"))
    }
}
