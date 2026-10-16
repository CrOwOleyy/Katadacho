pub mod audio;
pub mod bluetooth;
pub mod brightness;
pub mod radios;
pub mod wifi;
pub mod policy_config;

use crate::com::ComGuard;
use audio::AudioState;
use serde::Serialize;
use windows::Devices::Radios::RadioKind;

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

#[tauri::command]
pub async fn get_brightness() -> Result<u8, String> {
    blocking(brightness::get_brightness).await
}

#[tauri::command]
pub async fn set_brightness(level: u8) -> Result<(), String> {
    blocking(move || brightness::set_brightness(level)).await
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetworkBluetoothState {
    pub wifi_enabled: Option<bool>,
    pub wifi_ssid: Option<String>,
    pub wifi_signal: Option<u8>,
    pub bluetooth_enabled: Option<bool>,
    pub bluetooth_devices: Vec<String>,
}

#[tauri::command]
pub async fn get_network_bt_state() -> Result<NetworkBluetoothState, String> {
    blocking(|| {
        let _com = ComGuard::new().map_err(|e| e.to_string())?;
        let wifi_enabled = radios::is_enabled(RadioKind::WiFi).map_err(|e| e.to_string())?;
        let bluetooth_enabled = radios::is_enabled(RadioKind::Bluetooth).map_err(|e| e.to_string())?;
        // Pas de connexion Wi-Fi n'est pas une erreur : on affiche simplement vide.
        let connection = wifi::current_connection().unwrap_or_default();
        let bluetooth_devices = bluetooth::connected_device_names().map_err(|e| e.to_string())?;

        Ok(NetworkBluetoothState {
            wifi_enabled,
            wifi_ssid: connection.ssid,
            wifi_signal: connection.signal,
            bluetooth_enabled,
            bluetooth_devices,
        })
    })
    .await
}

#[tauri::command]
pub async fn toggle_wifi(enable: bool) -> Result<(), String> {
    blocking(move || {
        let _com = ComGuard::new().map_err(|e| e.to_string())?;
        radios::set_enabled(RadioKind::WiFi, enable).map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
pub async fn toggle_bluetooth(enable: bool) -> Result<(), String> {
    blocking(move || {
        let _com = ComGuard::new().map_err(|e| e.to_string())?;
        radios::set_enabled(RadioKind::Bluetooth, enable).map_err(|e| e.to_string())
    })
    .await
}
