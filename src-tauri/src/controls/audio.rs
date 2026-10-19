use super::types::{AudioDevice, AudioState};
use windows::core::{Result as WinResult, PWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eConsole, eRender, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoTaskMemFree, CLSCTX_ALL,
    STGM_READ,
};
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;

fn enumerator() -> WinResult<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
}

/// Convertit un PWSTR alloué par COM en String puis libère la mémoire.
fn take_pwstr(id: PWSTR) -> WinResult<String> {
    let text = unsafe { id.to_string() }.map_err(|_| windows::core::Error::from(windows::Win32::Foundation::E_FAIL))?;
    unsafe { CoTaskMemFree(Some(id.0 as *const std::ffi::c_void)) };
    Ok(text)
}

fn device_id(device: &IMMDevice) -> WinResult<String> {
    take_pwstr(unsafe { device.GetId()? })
}

fn device_name(device: &IMMDevice) -> WinResult<String> {
    let store: IPropertyStore = unsafe { device.OpenPropertyStore(STGM_READ)? };
    let value = unsafe { store.GetValue(&PKEY_Device_FriendlyName)? };
    let pwstr = unsafe { PropVariantToStringAlloc(&value)? };
    take_pwstr(pwstr)
}

fn endpoint_volume(device: &IMMDevice) -> WinResult<IAudioEndpointVolume> {
    unsafe { device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) }
}

fn default_device(enumerator: &IMMDeviceEnumerator) -> WinResult<IMMDevice> {
    unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole) }
}

/// Liste les périphériques de sortie actifs, avec le périphérique par défaut marqué.
pub fn list_render_devices() -> WinResult<Vec<AudioDevice>> {
    let enumerator = enumerator()?;
    let default_id = device_id(&default_device(&enumerator)?)?;
    let collection = unsafe { enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)? };
    let count = unsafe { collection.GetCount()? };

    let mut devices = Vec::with_capacity(count as usize);
    for index in 0..count {
        let device = unsafe { collection.Item(index)? };
        let id = device_id(&device)?;
        let name = device_name(&device).unwrap_or_else(|_| id.clone());
        let is_default = id == default_id;
        devices.push(AudioDevice { id, name, is_default });
    }
    Ok(devices)
}

pub fn get_state() -> WinResult<AudioState> {
    let enumerator = enumerator()?;
    let device = default_device(&enumerator)?;
    let volume = endpoint_volume(&device)?;

    let level = unsafe { volume.GetMasterVolumeLevelScalar()? };
    let muted = unsafe { volume.GetMute()? }.as_bool();

    let devices = list_render_devices()?;
    let active_device = devices.iter().find(|d| d.is_default).cloned();

    Ok(AudioState {
        volume: (level * 100.0).round().clamp(0.0, 100.0) as u8,
        muted,
        active_device,
        devices,
    })
}

pub fn set_master_volume(level: u8) -> WinResult<()> {
    let enumerator = enumerator()?;
    let volume = endpoint_volume(&default_device(&enumerator)?)?;
    let scalar = f32::from(level.min(100)) / 100.0;
    unsafe { volume.SetMasterVolumeLevelScalar(scalar, std::ptr::null()) }
}

/// Inverse le mute et retourne le nouvel état.
pub fn toggle_mute() -> WinResult<bool> {
    let enumerator = enumerator()?;
    let volume = endpoint_volume(&default_device(&enumerator)?)?;
    let next = !unsafe { volume.GetMute()? }.as_bool();
    unsafe { volume.SetMute(next.into(), std::ptr::null())? };
    Ok(next)
}
