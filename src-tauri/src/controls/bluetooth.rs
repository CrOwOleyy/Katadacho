//! Appareils Bluetooth actuellement connectés, via `DeviceInformation` (WinRT).

use windows::core::Result as WinResult;
use windows::Devices::Bluetooth::{BluetoothConnectionStatus, BluetoothDevice};
use windows::Devices::Enumeration::DeviceInformation;

/// Noms lisibles des appareils Bluetooth connectés, sans doublon.
pub fn connected_device_names() -> WinResult<Vec<String>> {
    let selector =
        BluetoothDevice::GetDeviceSelectorFromConnectionStatus(BluetoothConnectionStatus::Connected)?;
    let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.join()?;

    let mut names: Vec<String> = Vec::new();
    for index in 0..devices.Size()? {
        let name = devices.GetAt(index)?.Name()?.to_string();
        if !name.is_empty() && !names.contains(&name) {
            names.push(name);
        }
    }
    Ok(names)
}
