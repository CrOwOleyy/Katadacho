use std::ffi::c_void;
use std::ptr;

use serde::Serialize;
use windows::core::{Error, Result as WinResult, GUID};
use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE};
use windows::Win32::NetworkManagement::WiFi::{
    WlanCloseHandle, WlanEnumInterfaces, WlanFreeMemory, WlanOpenHandle, WlanQueryInterface,
    WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO_LIST,
    WLAN_INTERFACE_STATE, wlan_intf_opcode_current_connection,
};

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct WifiConnection {
    pub ssid: Option<String>,
    pub signal: Option<u8>,
}

/// Ferme le handle WLAN à la fin du scope, même en cas d'erreur.
struct WlanHandle(HANDLE);

impl Drop for WlanHandle {
    fn drop(&mut self) {
        unsafe { WlanCloseHandle(self.0, None) };
    }
}

fn open() -> WinResult<WlanHandle> {
    let mut negotiated: u32 = 0;
    let mut handle = HANDLE::default();
    let status = unsafe { WlanOpenHandle(2, None, &mut negotiated, &mut handle) };
    if status != ERROR_SUCCESS.0 {
        return Err(Error::from_hresult(windows::core::HRESULT::from_win32(status)));
    }
    Ok(WlanHandle(handle))
}

/// Retourne le SSID et la qualité du signal de la première interface Wi-Fi connectée.
/// Si aucune interface n'est connectée, les deux champs sont `None`.
pub fn current_connection() -> WinResult<WifiConnection> {
    let handle = open()?;

    let mut list_ptr: *mut WLAN_INTERFACE_INFO_LIST = ptr::null_mut();
    let status = unsafe { WlanEnumInterfaces(handle.0, None, &mut list_ptr) };
    if status != ERROR_SUCCESS.0 || list_ptr.is_null() {
        return Err(Error::from_hresult(windows::core::HRESULT::from_win32(status)));
    }

    let result = unsafe {
        let list = &*list_ptr;
        let interfaces = std::slice::from_raw_parts(
            list.InterfaceInfo.as_ptr(),
            list.dwNumberOfItems as usize,
        );

        let connected = interfaces
            .iter()
            .find(|info| info.isState == WLAN_INTERFACE_STATE(1));

        match connected {
            Some(info) => query_connection(handle.0, &info.InterfaceGuid),
            None => Ok(WifiConnection::default()),
        }
    };

    unsafe { WlanFreeMemory(list_ptr as *const c_void) };
    result
}

fn query_connection(handle: HANDLE, guid: &GUID) -> WinResult<WifiConnection> {
    let mut size: u32 = 0;
    let mut data: *mut c_void = ptr::null_mut();
    let status = unsafe {
        WlanQueryInterface(
            handle,
            guid,
            wlan_intf_opcode_current_connection,
            None,
            &mut size,
            &mut data,
            None,
        )
    };
    if status != ERROR_SUCCESS.0 || data.is_null() {
        return Ok(WifiConnection::default());
    }

    let attributes = unsafe { &*(data as *const WLAN_CONNECTION_ATTRIBUTES) };
    let association = &attributes.wlanAssociationAttributes;
    let ssid_len = association.dot11Ssid.uSSIDLength as usize;
    let ssid = String::from_utf8_lossy(&association.dot11Ssid.ucSSID[..ssid_len.min(32)]).into_owned();

    let connection = WifiConnection {
        ssid: (!ssid.is_empty()).then_some(ssid),
        signal: Some(association.wlanSignalQuality.min(100) as u8),
    };

    unsafe { WlanFreeMemory(data as *const c_void) };
    Ok(connection)
}
