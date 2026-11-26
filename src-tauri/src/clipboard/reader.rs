//! Lecture du presse-papier Windows : texte Unicode et images bitmap (CF_DIB).

use std::thread::sleep;
use std::time::Duration;

use image::RgbaImage;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{CF_DIB, CF_UNICODETEXT};

use super::ingest::ClipboardContent;

const OPEN_ATTEMPTS: u32 = 10;
const OPEN_RETRY_DELAY: Duration = Duration::from_millis(15);
const BI_RGB: u32 = 0;
const BI_BITFIELDS: u32 = 3;
const BITMAP_INFO_HEADER_MIN: usize = 40;


/// Ferme le presse-papier à la fin du scope. Un autre programme peut le verrouiller
/// brièvement, d'où les tentatives répétées à l'ouverture.
struct ClipboardGuard;

impl ClipboardGuard {
    fn open(owner: Option<HWND>) -> Option<Self> {
        for _ in 0..OPEN_ATTEMPTS {
            if unsafe { OpenClipboard(owner) }.is_ok() {
                return Some(Self);
            }
            sleep(OPEN_RETRY_DELAY);
        }
        None
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

/// Lit le contenu actuel. Le texte est prioritaire sur l'image, car beaucoup d'applications
/// (tableurs, navigateurs) fournissent les deux formats pour une même copie.
fn format_available(name: PCWSTR) -> Option<u32> {
    let format = unsafe { RegisterClipboardFormatW(name) };
    (format != 0 && unsafe { IsClipboardFormatAvailable(format) }.is_ok()).then_some(format)
}

/// Vrai si l'application qui a copié demande de ne pas historiser ce contenu. Les gestionnaires
/// de mots de passe (KeePass, Bitwarden, 1Password…) posent ces marqueurs sur leurs copies.
fn is_excluded_by_source() -> bool {
    if format_available(w!("ExcludeClipboardContentFromMonitorProcessing")).is_some()
        || format_available(w!("Clipboard Viewer Ignore")).is_some()
    {
        return true;
    }

    // « CanIncludeInClipboardHistory » contient un DWORD : 0 signifie « ne pas historiser ».
    let Some(format) = format_available(w!("CanIncludeInClipboardHistory")) else {
        return false;
    };
    let Ok(handle) = (unsafe { GetClipboardData(format) }) else {
        return false;
    };
    let memory = HGLOBAL(handle.0);
    let pointer = unsafe { GlobalLock(memory) } as *const u32;
    if pointer.is_null() {
        return false;
    }
    let allowed = unsafe { GlobalSize(memory) } >= 4 && unsafe { pointer.read_unaligned() } != 0;
    let _ = unsafe { GlobalUnlock(memory) };
    !allowed
}

pub fn read(owner: Option<HWND>) -> Option<ClipboardContent> {
    let _guard = ClipboardGuard::open(owner)?;

    if is_excluded_by_source() {
        return None;
    }

    if unsafe { IsClipboardFormatAvailable(u32::from(CF_UNICODETEXT.0)) }.is_ok() {
        return read_text().map(ClipboardContent::Text);
    }
    if unsafe { IsClipboardFormatAvailable(u32::from(CF_DIB.0)) }.is_ok() {
        return read_dib().map(ClipboardContent::Image);
    }
    None
}

fn read_text() -> Option<String> {
    let handle = unsafe { GetClipboardData(u32::from(CF_UNICODETEXT.0)) }.ok()?;
    let memory = HGLOBAL(handle.0);

    let pointer = unsafe { GlobalLock(memory) } as *const u16;
    if pointer.is_null() {
        return None;
    }

    let capacity = unsafe { GlobalSize(memory) } / 2;
    let units = unsafe { std::slice::from_raw_parts(pointer, capacity) };
    let length = units.iter().position(|&unit| unit == 0).unwrap_or(units.len());
    let text = String::from_utf16_lossy(&units[..length]);

    let _ = unsafe { GlobalUnlock(memory) };
    Some(text)
}

fn read_dib() -> Option<RgbaImage> {
    let handle = unsafe { GetClipboardData(u32::from(CF_DIB.0)) }.ok()?;
    let memory = HGLOBAL(handle.0);

    let pointer = unsafe { GlobalLock(memory) } as *const u8;
    if pointer.is_null() {
        return None;
    }
    let size = unsafe { GlobalSize(memory) };
    let bytes = unsafe { std::slice::from_raw_parts(pointer, size) };

    let image = decode_dib(bytes);

    let _ = unsafe { GlobalUnlock(memory) };
    image
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    bytes.get(offset..offset + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Décode un DIB 24 ou 32 bits (BI_RGB ou BI_BITFIELDS). Les formats à palette (8 bits et moins)
/// sont ignorés : ils sont devenus très rares dans les copies d'écran modernes.
fn decode_dib(bytes: &[u8]) -> Option<RgbaImage> {
    if bytes.len() < BITMAP_INFO_HEADER_MIN {
        return None;
    }

    let header_size = u32_at(bytes, 0)? as usize;
    let width = u32_at(bytes, 4)? as i32;
    let height = u32_at(bytes, 8)? as i32;
    let bit_count = u16::from_le_bytes([bytes[14], bytes[15]]);
    let compression = u32_at(bytes, 16)?;

    if width <= 0 || height == 0 || !(bit_count == 24 || bit_count == 32) {
        return None;
    }
    if compression != BI_RGB && compression != BI_BITFIELDS {
        return None;
    }

    // Avec BI_BITFIELDS et un en-tête de 40 octets, trois masques de 4 octets suivent l'en-tête.
    let masks = if compression == BI_BITFIELDS && header_size == BITMAP_INFO_HEADER_MIN { 12 } else { 0 };
    let pixel_offset = header_size + masks;

    let width = width as usize;
    let rows = height.unsigned_abs() as usize;
    let bottom_up = height > 0;
    let bytes_per_pixel = usize::from(bit_count / 8);
    let stride = (width * usize::from(bit_count)).div_ceil(32) * 4;

    let pixels = bytes.get(pixel_offset..pixel_offset + stride * rows)?;

    let mut rgba = vec![0u8; width * rows * 4];
    let mut any_alpha = false;

    for row in 0..rows {
        let source_row = if bottom_up { rows - 1 - row } else { row };
        let source = &pixels[source_row * stride..source_row * stride + width * bytes_per_pixel];
        let target = &mut rgba[row * width * 4..(row + 1) * width * 4];

        for column in 0..width {
            let pixel = &source[column * bytes_per_pixel..(column + 1) * bytes_per_pixel];
            let alpha = if bytes_per_pixel == 4 { pixel[3] } else { 255 };
            any_alpha |= alpha != 0;
            target[column * 4] = pixel[2];
            target[column * 4 + 1] = pixel[1];
            target[column * 4 + 2] = pixel[0];
            target[column * 4 + 3] = alpha;
        }
    }

    // Les captures d'écran 32 bits laissent souvent l'alpha à zéro : on force l'opacité.
    if !any_alpha {
        for pixel in rgba.chunks_exact_mut(4) {
            pixel[3] = 255;
        }
    }

    RgbaImage::from_raw(width as u32, rows as u32, rgba)
}
