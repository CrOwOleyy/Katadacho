pub mod collector;
pub mod processes;

use tauri::AppHandle;

/// Démarre l'émission de `system-stats` chaque seconde. Sans effet si elle tourne déjà.
#[tauri::command]
pub fn start_system_stats(app: AppHandle) {
    collector::start(app);
}

/// Arrête l'émission, par exemple à la fermeture du tableau de bord.
#[tauri::command]
pub fn stop_system_stats() {
    collector::stop();
}

#[tauri::command]
pub async fn kill_process(pid: u32) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || processes::kill(pid))
        .await
        .map_err(|e| e.to_string())?
}
