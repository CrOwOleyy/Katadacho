mod controls;
mod window;

use tauri::Manager;

use window::click_through::{self, ClickThroughState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(ClickThroughState::default())
        .setup(|app| {
            window::position_island(app)?;
            if let Some(island) = app.get_webview_window(window::ISLAND_LABEL) {
                island.set_ignore_cursor_events(true)?;
            }
            click_through::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            click_through::update_island_bounds,
            controls::get_audio_state,
            controls::set_master_volume,
            controls::toggle_mute,
            controls::set_default_audio_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
