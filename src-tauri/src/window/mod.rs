pub mod click_through;
#[cfg(target_os = "linux")]
pub mod layer_shell;

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

/// Prépare la fenêtre de l'Island au démarrage. Sous Wayland, quand le compositeur le permet, elle
/// devient une surface layer-shell dont seule la pilule reçoit la souris. Sinon (Windows, X11,
/// GNOME), elle est placée en haut de l'écran et rendue traversable, et un thread suit le curseur.
pub fn prepare_island(app: &App) -> tauri::Result<()> {
    let Some(island) = app.get_webview_window(ISLAND_LABEL) else {
        return Ok(());
    };

    #[cfg(target_os = "linux")]
    if layer_shell::install(&island) {
        return Ok(());
    }

    position_island(app)?;
    island.set_ignore_cursor_events(true)?;
    click_through::start(app.handle().clone());
    Ok(())
}
