//! Icône dans la zone de notification de Windows : seul moyen de quitter l'application,
//! puisque l'Island n'a pas de fenêtre dans la barre des tâches.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter};

/// Demande au frontend d'afficher l'Island déployée, éventuellement sur un onglet précis.
pub const EVENT_ISLAND_OPEN: &str = "island-open";

fn open_island(app: &AppHandle, tab: Option<&str>) {
    let _ = app.emit(EVENT_ISLAND_OPEN, tab);
}

pub fn create(app: &App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open the island", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let restart = MenuItem::with_id(app, "restart", "Restart", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Katadacho", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &settings, &separator, &restart, &quit])?;

    let mut builder = TrayIconBuilder::with_id("katadacho")
        .tooltip("Katadacho")
        .menu(&menu)
        // Le clic gauche ouvre l'Island, le clic droit affiche le menu.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => open_island(app, None),
            "settings" => open_island(app, Some("settings")),
            "restart" => app.restart(),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                open_island(tray.app_handle(), None);
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}
