//! Remet un élément de l'historique dans le presse-papier Windows, puis colle éventuellement
//! dans la fenêtre précédemment active en simulant `Ctrl+V`.

use std::mem::size_of;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::System::Ole::{CF_DIB, CF_UNICODETEXT};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL,
};
use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;

const OPEN_ATTEMPTS: u32 = 10;
const OPEN_RETRY_DELAY: Duration = Duration::from_millis(15);
/// Laisse à Windows le temps de basculer le focus avant d'envoyer les touches.
const FOCUS_SETTLE_DELAY: Duration = Duration::from_millis(80);
const VK_V: VIRTUAL_KEY = VIRTUAL_KEY(0x56);
const DIB_HEADER_SIZE: usize = 40;

fn open_clipboard() -> Result<(), String> {
    for _ in 0..OPEN_ATTEMPTS {
        if unsafe { OpenClipboard(None) }.is_ok() {
            return Ok(());
        }
        sleep(OPEN_RETRY_DELAY);
    }
    Err("Le presse-papier est verrouillé par une autre application".to_string())
}

/// Copie `bytes` dans une mémoire globale et la donne au presse-papier sous `format`.
/// Après `SetClipboardData`, Windows possède la mémoire : on ne la libère pas.
fn set_clipboard_bytes(format: u16, bytes: &[u8]) -> Result<(), String> {
    open_clipboard()?;

    let result = (|| -> Result<(), String> {
        unsafe { EmptyClipboard() }.map_err(|e| e.to_string())?;

        let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes.len()) }.map_err(|e| e.to_string())?;
        let pointer = unsafe { GlobalLock(memory) } as *mut u8;
        if pointer.is_null() {
            return Err("Allocation mémoire impossible".to_string());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len());
            let _ = GlobalUnlock(memory);
            SetClipboardData(u32::from(format), Some(HANDLE(memory.0))).map_err(|e| e.to_string())?;
        }
        Ok(())
    })();

    let _ = unsafe { CloseClipboard() };
    result
}

pub fn put_text(text: &str) -> Result<(), String> {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes: Vec<u8> = wide.iter().flat_map(|unit| unit.to_le_bytes()).collect();
    set_clipboard_bytes(CF_UNICODETEXT.0, &bytes)
}

/// Charge un PNG et le place en DIB 32 bits, orienté du bas vers le haut comme l'exige Windows.
pub fn put_image(path: &Path) -> Result<(), String> {
    let image = image::open(path).map_err(|e| e.to_string())?.to_rgba8();
    let (width, height) = (image.width() as usize, image.height() as usize);

    let mut dib = Vec::with_capacity(DIB_HEADER_SIZE + width * height * 4);
    dib.extend_from_slice(&(DIB_HEADER_SIZE as u32).to_le_bytes());
    dib.extend_from_slice(&(width as i32).to_le_bytes());
    dib.extend_from_slice(&(height as i32).to_le_bytes());
    dib.extend_from_slice(&1u16.to_le_bytes());
    dib.extend_from_slice(&32u16.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&((width * height * 4) as u32).to_le_bytes());
    dib.extend_from_slice(&[0u8; 16]);

    for row in (0..height).rev() {
        for column in 0..width {
            let pixel = image.get_pixel(column as u32, row as u32);
            dib.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }

    set_clipboard_bytes(CF_DIB.0, &dib)
}

fn key_event(key: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { Default::default() },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Rend le focus à `window`, attend un court instant, puis envoie `Ctrl+V`.
pub fn paste_into(window: HWND) -> Result<(), String> {
    if !unsafe { SetForegroundWindow(window) }.as_bool() {
        return Err("Windows a refusé de rendre le focus à l'application précédente".to_string());
    }
    sleep(FOCUS_SETTLE_DELAY);

    let inputs = [
        key_event(VK_CONTROL, false),
        key_event(VK_V, false),
        key_event(VK_V, true),
        key_event(VK_CONTROL, true),
    ];
    let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err("Envoi de Ctrl+V incomplet".to_string())
    }
}
