//! Média en cours de lecture : titre, artiste, état de lecture et pochette.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, State};

pub mod controls;
mod cover;
#[cfg(windows)]
mod gsmtc;
pub mod levels;
#[cfg(target_os = "linux")]
mod mpris;

pub const EVENT_MEDIA_CHANGED: &str = "media-changed";

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub cover_base64: Option<String>,
    pub cover_mime: Option<String>,
}

/// Dernier état connu, pour que le frontend puisse le relire au montage sans attendre un événement.
#[derive(Default)]
pub struct MediaState(Mutex<Option<MediaInfo>>);

impl MediaState {
    #[cfg_attr(not(windows), allow(dead_code))]
    pub(crate) fn set(&self, info: Option<MediaInfo>) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = info;
    }
}

#[tauri::command]
pub fn get_media_state(state: State<'_, MediaState>) -> Option<MediaInfo> {
    state.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Lance la surveillance du média en cours. Windows : contrôles multimédias système. Linux : MPRIS.
/// Les autres systèmes ne signalent aucun média : la pilule reste simplement sans pochette.
pub fn start(app: AppHandle) {
    #[cfg(windows)]
    gsmtc::start(app);
    #[cfg(target_os = "linux")]
    mpris::start(app);
    #[cfg(not(any(windows, target_os = "linux")))]
    drop(app);
}
