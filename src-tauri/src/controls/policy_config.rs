//! Interface COM `IPolicyConfig` (non documentée par Microsoft).
//! Seule `SetDefaultEndpoint` est appelée. Les autres slots de la vtable sont
//! déclarés en `usize` pour conserver la disposition mémoire exacte.

use std::ffi::c_void;

use windows::core::{Interface, Result as WinResult, GUID, HRESULT, PCWSTR};
use windows::Win32::Media::Audio::{eCommunications, eConsole, eMultimedia, ERole};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

const CLSID_POLICY_CONFIG_CLIENT: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);
const IID_POLICY_CONFIG: GUID = GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);

#[repr(C)]
struct IPolicyConfigVtbl {
    query_interface: usize,
    add_ref: usize,
    release: usize,
    get_mix_format: usize,
    get_device_format: usize,
    set_device_format: usize,
    get_processing_period: usize,
    set_processing_period: usize,
    get_share_mode: usize,
    set_share_mode: usize,
    get_property_value: usize,
    set_property_value: usize,
    set_default_endpoint:
        unsafe extern "system" fn(this: *mut c_void, device_id: PCWSTR, role: ERole) -> HRESULT,
    set_endpoint_visibility: usize,
}

/// Définit le périphérique de sortie par défaut pour les 3 rôles Windows
/// (console, multimédia, communications), comme le fait la fenêtre Paramètres.
pub fn set_default_endpoint(device_id: &str) -> WinResult<()> {
    let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
    let policy: windows::core::IUnknown =
        unsafe { CoCreateInstance(&CLSID_POLICY_CONFIG_CLIENT, None, CLSCTX_ALL)? };

    let mut queried: *mut c_void = std::ptr::null_mut();
    unsafe { policy.query(&IID_POLICY_CONFIG, &mut queried) }.ok()?;
    // Prend la référence ajoutée par QueryInterface, libérée au drop.
    let config = unsafe { windows::core::IUnknown::from_raw(queried) };
    let raw = config.as_raw();
    let vtbl = unsafe { *(raw as *const *const IPolicyConfigVtbl) };
    let set_default = unsafe { (*vtbl).set_default_endpoint };

    for role in [eConsole, eMultimedia, eCommunications] {
        let hr = unsafe { set_default(raw, PCWSTR(wide.as_ptr()), role) };
        hr.ok()?;
    }
    Ok(())
}
