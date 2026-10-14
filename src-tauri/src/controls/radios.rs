//! Radios Wi-Fi et Bluetooth via `Windows.Devices.Radios` (WinRT).

use windows::core::{Error, Result as WinResult};
use windows::Devices::Radios::{Radio, RadioAccessStatus, RadioKind, RadioState};
use windows::Win32::Foundation::{E_ACCESSDENIED, E_FAIL};

fn find_radio(kind: RadioKind) -> WinResult<Option<Radio>> {
    // Accès obligatoire avant toute lecture ou écriture de radio.
    let access = Radio::RequestAccessAsync()?.join()?;
    if access != RadioAccessStatus::Allowed {
        return Err(Error::from(E_ACCESSDENIED));
    }

    let radios = Radio::GetRadiosAsync()?.join()?;
    for index in 0..radios.Size()? {
        let radio = radios.GetAt(index)?;
        if radio.Kind()? == kind {
            return Ok(Some(radio));
        }
    }
    Ok(None)
}

/// `Some(true|false)` si la radio existe sur le système, `None` si elle est absente.
pub fn is_enabled(kind: RadioKind) -> WinResult<Option<bool>> {
    match find_radio(kind)? {
        Some(radio) => Ok(Some(radio.State()? == RadioState::On)),
        None => Ok(None),
    }
}

pub fn set_enabled(kind: RadioKind, enable: bool) -> WinResult<()> {
    let radio = find_radio(kind)?.ok_or_else(|| Error::from(E_FAIL))?;
    let target = if enable { RadioState::On } else { RadioState::Off };
    match radio.SetStateAsync(target)?.join()? {
        RadioAccessStatus::Allowed => Ok(()),
        _ => Err(Error::from(E_ACCESSDENIED)),
    }
}
