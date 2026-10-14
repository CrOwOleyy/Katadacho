pub mod audio;
pub mod policy_config;

use audio::{AudioState, ComGuard};

async fn blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    match tauri::async_runtime::spawn_blocking(task).await {
        Ok(result) => result,
        Err(join_error) => Err(join_error.to_string()),
    }
}

#[tauri::command]
pub async fn get_audio_state() -> Result<AudioState, String> {
    blocking(|| {
        let _com = ComGuard::new().map_err(|e| e.to_string())?;
        audio::get_state().map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
pub async fn set_master_volume(level: u8) -> Result<(), String> {
    blocking(move || {
        let _com = ComGuard::new().map_err(|e| e.to_string())?;
        audio::set_master_volume(level).map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
pub async fn toggle_mute() -> Result<bool, String> {
    blocking(|| {
        let _com = ComGuard::new().map_err(|e| e.to_string())?;
        audio::toggle_mute().map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
pub async fn set_default_audio_device(device_id: String) -> Result<(), String> {
    blocking(move || {
        let _com = ComGuard::new().map_err(|e| e.to_string())?;
        policy_config::set_default_endpoint(&device_id).map_err(|e| e.to_string())
    })
    .await
}
