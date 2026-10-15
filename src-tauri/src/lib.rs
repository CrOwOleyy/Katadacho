mod clipboard;
mod controls;
mod db;
mod launcher;
mod media;
mod settings;
mod state;
mod speedtest;
mod telemetry;
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
        .manage(launcher::Launcher::default())
        .manage(media::MediaState::default())
        .setup(|app| {
            let data_dir = app.path().local_data_dir()?.join("Katadacho");
            let database = db::Database::open(&data_dir)?;
            app.manage(settings::SettingsState(std::sync::RwLock::new(settings::load(&database))));
            app.manage(database);
            app.manage(state::AppPaths { data_dir });
            window::position_island(app)?;
            if let Some(island) = app.get_webview_window(window::ISLAND_LABEL) {
                island.set_ignore_cursor_events(true)?;
            }
            click_through::start(app.handle().clone());
            clipboard::listener::start(app.handle().clone());
            clipboard::foreground::start();
            launcher::start_indexing(app.handle().clone());
            launcher::hotkey::register(app.handle());
            media::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            click_through::update_island_bounds,
            controls::get_audio_state,
            controls::set_master_volume,
            controls::toggle_mute,
            controls::set_default_audio_device,
            controls::get_brightness,
            controls::set_brightness,
            controls::get_network_bt_state,
            controls::toggle_wifi,
            controls::toggle_bluetooth,
            telemetry::start_system_stats,
            telemetry::stop_system_stats,
            telemetry::kill_process,
            speedtest::run_speedtest,
            clipboard::commands::get_clipboard_history,
            clipboard::commands::toggle_pin_clipboard,
            clipboard::commands::delete_clipboard_item,
            clipboard::commands::paste_clipboard_item,
            launcher::commands::search_launcher,
            launcher::commands::launch_item,
            launcher::commands::open_item_location,
            launcher::commands::reindex_launcher,
            launcher::commands::get_launcher_icon,
            media::get_media_state,
            media::levels::start_audio_levels,
            media::levels::stop_audio_levels,
            media::controls::media_toggle_play_pause,
            media::controls::media_next,
            media::controls::media_previous,
            media::controls::media_seek,
            media::controls::media_position,
            settings::get_settings,
            settings::save_settings,
            settings::reset_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
