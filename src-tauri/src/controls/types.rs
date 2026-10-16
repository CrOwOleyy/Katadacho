//! Données des contrôles système, identiques sur tous les systèmes.

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudioState {
    pub volume: u8,
    pub muted: bool,
    pub active_device: Option<AudioDevice>,
    pub devices: Vec<AudioDevice>,
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
