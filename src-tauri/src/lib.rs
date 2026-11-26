mod clipboard;
#[cfg(windows)]
mod com;
mod controls;
mod db;
mod app_context;
mod fullscreen;
mod icon_theme;
mod idle;
mod launcher;
pub mod mcp;
mod media;
mod platform;
pub mod memory;
mod notifications;
mod settings;
mod sound;
mod state;
mod speedtest;
mod telemetry;
mod timer;
mod tray;
mod window;

use tauri::{Emitter, Manager};

use window::click_through::{self, ClickThroughState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        // Relancer Katadacho alors qu'il tourne déjà ouvre l'Island de l'instance en cours, au lieu de
        // ne rien faire de visible.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.emit(tray::EVENT_ISLAND_OPEN, None::<&str>);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(ClickThroughState::default())
        .manage(launcher::Launcher::default())
        .manage(media::MediaState::default())
        .manage(timer::TimerState::default())
        .manage(mcp::bridge::PendingQuestions::default())
        .manage(launcher::hotkey::HotkeyState::default())
        .setup(|app| {
            let data_dir = app.path().local_data_dir()?.join("Katadacho");
            let database = db::Database::open(&data_dir)?;
            app.manage(settings::SettingsState(std::sync::RwLock::new(settings::load(&database))));
            app.manage(database);
            mcp::bridge::start(app.handle().clone(), data_dir.clone());
            app.manage(state::AppPaths { data_dir });
            tray::create(app)?;
            window::prepare_island(app)?;
            clipboard::start(app.handle().clone());
            launcher::start_indexing(app.handle().clone());
            let launcher_hotkey = app.state::<settings::SettingsState>().snapshot().hotkeys.launcher;
            launcher::hotkey::apply(app.handle(), &launcher_hotkey);
            media::start(app.handle().clone());
            timer::start(app.handle().clone());
            notifications::start(app.handle().clone());
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
            controls::get_app_volumes,
            controls::set_app_volume,
            controls::toggle_app_mute,
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
            launcher::hotkey::get_hotkey_status,
            idle::get_idle_seconds,
            mcp::get_claude_bridge_status,
            mcp::install_claude_mcp_bridge,
            mcp::uninstall_claude_mcp_bridge,
            mcp::answer_ai_question,
            app_context::get_foreground_app,
            fullscreen::is_fullscreen_app_active,
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
            memory::clean_memory_quick,
            memory::clean_memory_deep,
            timer::timer_get,
            timer::timer_start_pomodoro,
            timer::timer_start_countdown,
            timer::timer_pause,
            timer::timer_resume,
            timer::timer_reset,
            timer::timer_skip,
            notifications::get_notification_access,
            notifications::request_notification_access,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
