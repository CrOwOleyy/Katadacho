#[cfg(not(windows))]
mod linux;
mod mixer;
mod parse;
#[cfg(windows)]
pub mod audio;
#[cfg(windows)]
pub mod bluetooth;
#[cfg(windows)]
pub mod brightness;
#[cfg(windows)]
pub mod policy_config;
#[cfg(windows)]
pub mod radios;
pub mod types;
#[cfg(windows)]
pub mod wifi;

pub use types::{AppVolume, AudioState, NetworkBluetoothState};

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

/// Implémentation Windows : CoreAudio, WMI et DDC/CI, WLAN et WinRT.
#[cfg(windows)]
mod imp {
    use windows::Devices::Radios::RadioKind;

    use super::types::{AudioState, NetworkBluetoothState};
    use super::{audio, bluetooth, brightness, policy_config, radios, wifi};
    use crate::com::ComGuard;

    fn com() -> Result<ComGuard, String> {
        ComGuard::new().map_err(|e| e.to_string())
    }

    pub fn get_audio_state() -> Result<AudioState, String> {
        let _com = com()?;
        audio::get_state().map_err(|e| e.to_string())
    }

    pub fn set_master_volume(level: u8) -> Result<(), String> {
        let _com = com()?;
        audio::set_master_volume(level).map_err(|e| e.to_string())
    }

    pub fn toggle_mute() -> Result<bool, String> {
        let _com = com()?;
        audio::toggle_mute().map_err(|e| e.to_string())
    }

    pub fn set_default_audio_device(device_id: &str) -> Result<(), String> {
        let _com = com()?;
        policy_config::set_default_endpoint(device_id).map_err(|e| e.to_string())
    }

    pub fn get_brightness() -> Result<u8, String> {
        brightness::get_brightness()
    }

    pub fn set_brightness(level: u8) -> Result<(), String> {
        brightness::set_brightness(level)
    }

    pub fn get_network_bt_state() -> Result<NetworkBluetoothState, String> {
        let _com = com()?;
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
    }

    pub fn toggle_wifi(enable: bool) -> Result<(), String> {
        let _com = com()?;
        radios::set_enabled(RadioKind::WiFi, enable).map_err(|e| e.to_string())
    }

    pub fn toggle_bluetooth(enable: bool) -> Result<(), String> {
        let _com = com()?;
        radios::set_enabled(RadioKind::Bluetooth, enable).map_err(|e| e.to_string())
    }
}

/// Linux : outils en ligne de commande standard (pactl, nmcli, bluetoothctl, logind).
#[cfg(not(windows))]
mod imp {
    pub use super::linux::*;
}

#[tauri::command]
pub async fn get_audio_state() -> Result<AudioState, String> {
    blocking(imp::get_audio_state).await
}

#[tauri::command]
pub async fn set_master_volume(level: u8) -> Result<(), String> {
    blocking(move || imp::set_master_volume(level)).await
}

#[tauri::command]
pub async fn toggle_mute() -> Result<bool, String> {
    blocking(imp::toggle_mute).await
}

#[tauri::command]
pub async fn set_default_audio_device(device_id: String) -> Result<(), String> {
    blocking(move || imp::set_default_audio_device(&device_id)).await
}

#[tauri::command]
pub async fn get_brightness() -> Result<u8, String> {
    blocking(imp::get_brightness).await
}

#[tauri::command]
pub async fn set_brightness(level: u8) -> Result<(), String> {
    blocking(move || imp::set_brightness(level)).await
}

#[tauri::command]
pub async fn get_network_bt_state() -> Result<NetworkBluetoothState, String> {
    blocking(imp::get_network_bt_state).await
}

#[tauri::command]
pub async fn toggle_wifi(enable: bool) -> Result<(), String> {
    blocking(move || imp::toggle_wifi(enable)).await
}

#[tauri::command]
pub async fn toggle_bluetooth(enable: bool) -> Result<(), String> {
    blocking(move || imp::toggle_bluetooth(enable)).await
}
