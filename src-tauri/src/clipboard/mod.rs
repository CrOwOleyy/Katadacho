#[cfg(windows)]
pub mod capture;
pub mod classify;
pub mod ingest;
#[cfg(target_os = "linux")]
pub mod linux;
pub mod commands;
#[cfg(windows)]
pub mod foreground;
#[cfg(windows)]
pub mod listener;
#[cfg(windows)]
pub mod paste;
#[cfg(windows)]
pub mod reader;
pub mod secrets;
#[cfg(windows)]
pub mod source;
pub mod store;

use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use tauri::{AppHandle, Emitter, Manager};

use crate::db::Database;
use crate::settings::SettingsState;
use store::ClipboardItem;

pub const EVENT_CLIPBOARD_UPDATED: &str = "clipboard-updated";

/// Démarre le presse-papier. Sous Windows, les copies sont capturées via `WM_CLIPBOARDUPDATE` et la
/// fenêtre active est suivie pour le collage. Sous Linux, le contenu est relu régulièrement (X11 et
/// Wayland). Le nettoyage des secrets tourne partout.
pub fn start(app: AppHandle) {
    #[cfg(windows)]
    {
        listener::start(app.clone());
        foreground::start();
    }
    #[cfg(target_os = "linux")]
    linux::start(app.clone());
    start_secret_cleanup(app);
}

/// Chemin de la miniature d'une image : même nom avec le suffixe `_thumb`.
pub fn thumbnail_path(image_path: &Path) -> PathBuf {
    let stem = image_path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    image_path.with_file_name(format!("{stem}_thumb.png"))
}

/// Ajoute la miniature en base64 aux éléments image, lue depuis le disque.
pub fn attach_thumbnail(item: &mut ClipboardItem) {
    let Some(path) = item.image_path.as_deref() else {
        return;
    };
    if let Ok(bytes) = fs::read(thumbnail_path(Path::new(path))) {
        item.thumbnail_base64 = Some(STANDARD.encode(bytes));
    }
}

/// Vérifie régulièrement les secrets à effacer.
const SECRET_SWEEP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(20);
const MS_PER_MINUTE: i64 = 60_000;

/// Efface en tâche de fond les secrets non épinglés plus vieux que la durée réglée.
pub fn start_secret_cleanup(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(SECRET_SWEEP_INTERVAL);

        let ttl_minutes = app.state::<SettingsState>().snapshot().clipboard.secret_ttl_minutes;
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis() as i64)
            .unwrap_or(0);
        let cutoff = now_ms - i64::from(ttl_minutes) * MS_PER_MINUTE;

        match store::delete_expired_sensitive(&app.state::<Database>(), cutoff) {
            Ok(0) => {}
            Ok(_) => {
                let _ = app.emit(EVENT_CLIPBOARD_UPDATED, ());
            }
            Err(error) => eprintln!("secret cleanup failed: {error}"),
        }
    });
}
