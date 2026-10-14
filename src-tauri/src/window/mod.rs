pub mod click_through;

use tauri::{App, Manager};

pub const ISLAND_LABEL: &str = "island";

/// Centre la fenêtre island en haut du moniteur principal (y = 0).
pub fn position_island(app: &App) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window(ISLAND_LABEL) else {
        return Ok(());
    };
    let Some(monitor) = window.primary_monitor()? else {
        return Ok(());
    };

    let monitor_size = monitor.size();
    let monitor_origin = monitor.position();
    let window_size = window.outer_size()?;

    let x = monitor_origin.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
    let y = monitor_origin.y;

    window.set_position(tauri::PhysicalPosition::new(x, y))
}
