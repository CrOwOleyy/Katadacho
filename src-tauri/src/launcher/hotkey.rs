//! Raccourci global qui ouvre le lanceur depuis n'importe quelle application.
//! Le choix vient des réglages : touche Copilot, ou une combinaison classique.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::window::ISLAND_LABEL;

pub const EVENT_LAUNCHER_HOTKEY: &str = "launcher-hotkey";
pub const EVENT_HOTKEY_STATUS: &str = "hotkey-status";

/// Choix proposés dans les réglages. `copilot` passe par le crochet clavier, les autres par
/// le plugin de raccourcis globaux.
pub const CHOICES: &[&str] = &["copilot", "alt+space", "ctrl+alt+space", "ctrl+shift+space"];
pub const DEFAULT_CHOICE: &str = "alt+space";

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    pub choice: String,
    pub active: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct HotkeyState {
    /// Combinaison enregistrée auprès du plugin, à libérer avant d'en changer.
    registered: Mutex<Option<String>>,
    status: Mutex<HotkeyStatus>,
}

fn activate(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(ISLAND_LABEL) {
        // Le focus clavier est nécessaire pour taper dans le champ de recherche.
        let _ = window.set_focus();
    }
    let _ = app.emit(EVENT_LAUNCHER_HOTKEY, ());
}

/// Active le raccourci choisi et désactive le précédent. L'échec n'empêche pas l'application de
/// tourner : il est simplement signalé dans les réglages.
pub fn apply(app: &AppHandle, choice: &str) {
    let state = app.state::<HotkeyState>();

    if let Some(previous) = state.registered.lock().unwrap_or_else(|e| e.into_inner()).take() {
        let _ = app.global_shortcut().unregister(previous.as_str());
    }
    #[cfg(windows)]
    super::copilot::set_enabled(false);

    let status = if choice == "copilot" {
        enable_copilot_key(app)
    } else if CHOICES.contains(&choice) {
        let handle = app.clone();
        let result = app.global_shortcut().on_shortcut(choice, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                activate(&handle);
            }
        });
        match result {
            Ok(()) => {
                *state.registered.lock().unwrap_or_else(|e| e.into_inner()) = Some(choice.to_string());
                HotkeyStatus { choice: choice.to_string(), active: true, error: None }
            }
            Err(_) => HotkeyStatus {
                choice: choice.to_string(),
                active: false,
                error: Some("Ce raccourci est déjà utilisé par une autre application".to_string()),
            },
        }
    } else {
        HotkeyStatus {
            choice: choice.to_string(),
            active: false,
            error: Some("Raccourci inconnu".to_string()),
        }
    };

    *state.status.lock().unwrap_or_else(|e| e.into_inner()) = status.clone();
    let _ = app.emit(EVENT_HOTKEY_STATUS, status);
}

#[tauri::command]
pub fn get_hotkey_status(state: State<'_, HotkeyState>) -> HotkeyStatus {
    state.status.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// La touche Copilot passe par un crochet clavier Windows.
#[cfg(windows)]
fn enable_copilot_key(app: &AppHandle) -> HotkeyStatus {
    let handle = app.clone();
    super::copilot::start(move || activate(&handle));
    super::copilot::set_enabled(true);
    HotkeyStatus { choice: "copilot".to_string(), active: true, error: None }
}

#[cfg(not(windows))]
fn enable_copilot_key(_app: &AppHandle) -> HotkeyStatus {
    HotkeyStatus {
        choice: "copilot".to_string(),
        active: false,
        error: Some("La touche Copilot n'existe que sous Windows".to_string()),
    }
}
