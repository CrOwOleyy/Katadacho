//! Raccourci global qui ouvre le lanceur depuis n'importe quelle application.

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::window::ISLAND_LABEL;

pub const EVENT_LAUNCHER_HOTKEY: &str = "launcher-hotkey";

/// Alt+Space est souvent déjà pris (menu système des fenêtres, PowerToys Run). On essaie donc
/// des alternatives dans l'ordre jusqu'à en trouver une libre.
const CANDIDATES: &[&str] = &["alt+space", "ctrl+alt+space"];

fn activate(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(ISLAND_LABEL) {
        // Le focus clavier est nécessaire pour taper dans le champ de recherche.
        let _ = window.set_focus();
    }
    let _ = app.emit(EVENT_LAUNCHER_HOTKEY, ());
}

/// Enregistre le premier raccourci disponible et retourne son nom, ou `None` si tous sont pris.
pub fn register(app: &AppHandle) -> Option<&'static str> {
    for candidate in CANDIDATES {
        let handle = app.clone();
        let result = app.global_shortcut().on_shortcut(*candidate, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                activate(&handle);
            }
        });

        match result {
            Ok(()) => return Some(candidate),
            Err(error) => eprintln!("launcher hotkey {candidate} unavailable: {error}"),
        }
    }
    None
}
