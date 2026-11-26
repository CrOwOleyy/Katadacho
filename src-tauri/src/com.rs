//! Initialisation COM de Windows, partagée par tous les modules qui utilisent COM ou WinRT.

use windows::core::Result as WinResult;
use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

/// Initialise COM pour le thread courant et le libère au drop, si l'init a réussi.
pub struct ComGuard {
    should_uninit: bool,
}

impl ComGuard {
    pub fn new() -> WinResult<Self> {
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr.is_ok() {
            Ok(Self { should_uninit: true })
        } else if hr == RPC_E_CHANGED_MODE {
            Ok(Self { should_uninit: false })
        } else {
            Err(windows::core::Error::from(hr))
        }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.should_uninit {
            unsafe { CoUninitialize() };
        }
    }
}
