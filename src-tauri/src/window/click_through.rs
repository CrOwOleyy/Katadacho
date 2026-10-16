use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use tauri::{AppHandle, Manager, State, WebviewWindow};

use super::ISLAND_LABEL;

/// Marge ajoutée autour de la pilule pour éviter le clignotement au bord.
const HOVER_MARGIN_PX: f64 = 8.0;
/// Zone de proximité où le curseur est suivi à ~60 Hz.
const NEAR_MARGIN_PX: f64 = 120.0;
const NEAR_INTERVAL: Duration = Duration::from_millis(16);
const FAR_INTERVAL: Duration = Duration::from_millis(100);

/// Rectangle de l'Island en pixels logiques CSS, relatif à la fenêtre.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IslandBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Default)]
pub struct ClickThroughState {
    bounds: Mutex<Option<IslandBounds>>,
}

#[tauri::command]
pub fn update_island_bounds(state: State<'_, ClickThroughState>, bounds: IslandBounds) {
    let mut guard = state.bounds.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(bounds);
}

/// Lance le thread de suivi du curseur. Il bascule `ignore_cursor_events`
/// uniquement lorsque l'état change, donc aucun appel inutile à Tauri.
pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut ignoring = true;

        loop {
            let Some(window) = app.get_webview_window(ISLAND_LABEL) else {
                thread::sleep(FAR_INTERVAL);
                continue;
            };

            let bounds = {
                let state: State<ClickThroughState> = app.state();
                let guard = state.bounds.lock().unwrap_or_else(|e| e.into_inner());
                *guard
            };

            let (inside, near) = match bounds {
                Some(b) => cursor_relation(&window, b),
                None => (false, false),
            };

            let should_ignore = !inside;
            if should_ignore != ignoring && window.set_ignore_cursor_events(should_ignore).is_ok() {
                ignoring = should_ignore;
            }

            thread::sleep(if near { NEAR_INTERVAL } else { FAR_INTERVAL });
        }
    });
}

/// Retourne (curseur dans la pilule avec marge, curseur proche de la pilule).
fn cursor_relation(window: &WebviewWindow, bounds: IslandBounds) -> (bool, bool) {
    // Position du curseur en pixels physiques de l'écran, fournie par Tauri sur tous les systèmes.
    let Ok(point) = window.cursor_position() else {
        return (false, false);
    };

    let Ok(origin) = window.outer_position() else {
        return (false, false);
    };
    let scale = window.scale_factor().unwrap_or(1.0);

    let left = origin.x as f64 + bounds.x * scale;
    let top = origin.y as f64 + bounds.y * scale;
    let right = left + bounds.width * scale;
    let bottom = top + bounds.height * scale;

    let cx = point.x;
    let cy = point.y;

    let inside = cx >= left - HOVER_MARGIN_PX
        && cx <= right + HOVER_MARGIN_PX
        && cy >= top - HOVER_MARGIN_PX
        && cy <= bottom + HOVER_MARGIN_PX;

    let near = cx >= left - NEAR_MARGIN_PX
        && cx <= right + NEAR_MARGIN_PX
        && cy >= top - NEAR_MARGIN_PX
        && cy <= bottom + NEAR_MARGIN_PX;

    (inside, near)
}
