//! Capture des copies sous Windows : lecture du presse-papier puis enregistrement partagé.

use tauri::AppHandle;
use windows::Win32::Foundation::HWND;

use super::{ingest, reader, source};

/// Appelé par le listener à chaque copie.
pub fn capture(app: &AppHandle, owner: HWND) {
    let Some(content) = reader::read(Some(owner)) else {
        return;
    };
    ingest::ingest(app, content, source::foreground_app_name());
}
