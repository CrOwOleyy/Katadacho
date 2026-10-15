//! Icônes Windows des applications et fichiers, via `IShellItemImageFactory`.
//! Chaque icône est mise en cache sur le disque en PNG, puis renvoyée en base64.

use std::ffi::c_void;
use std::fs;
use std::mem::size_of;
use std::path::Path;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::RgbaImage;
use sha2::{Digest, Sha256};
use windows::core::HSTRING;
use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HBITMAP, HGDIOBJ,
};
use windows::Win32::System::Com::IBindCtx;
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY,
};

const ICON_SIZE: i32 = 64;

fn cache_file(cache_dir: &Path, path: &str) -> std::path::PathBuf {
    let digest = Sha256::digest(path.to_lowercase().as_bytes());
    let name: String = digest.iter().take(16).map(|byte| format!("{byte:02x}")).collect();
    cache_dir.join(format!("{name}.png"))
}

/// Icône de `path` en PNG base64. Le COM doit être initialisé sur le thread appelant.
pub fn icon_base64(path: &str, cache_dir: &Path) -> Result<String, String> {
    let file = cache_file(cache_dir, path);
    if let Ok(bytes) = fs::read(&file) {
        return Ok(STANDARD.encode(bytes));
    }

    let image = extract(path)?;
    fs::create_dir_all(cache_dir).map_err(|e| e.to_string())?;
    image.save(&file).map_err(|e| e.to_string())?;

    fs::read(&file).map(|bytes| STANDARD.encode(bytes)).map_err(|e| e.to_string())
}

fn extract(path: &str) -> Result<RgbaImage, String> {
    let factory: IShellItemImageFactory =
        unsafe { SHCreateItemFromParsingName(&HSTRING::from(path), None::<&IBindCtx>) }
            .map_err(|e| e.to_string())?;

    let flags = SIIGBF(SIIGBF_ICONONLY.0 | SIIGBF_BIGGERSIZEOK.0);
    let bitmap = unsafe { factory.GetImage(SIZE { cx: ICON_SIZE, cy: ICON_SIZE }, flags) }
        .map_err(|e| e.to_string())?;

    let result = bitmap_to_image(bitmap);
    let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
    result
}

/// Lit les pixels d'un HBITMAP en RGBA. Le Shell renvoie des pixels BGRA pré-multipliés par l'alpha.
fn bitmap_to_image(bitmap: HBITMAP) -> Result<RgbaImage, String> {
    let mut info = BITMAP::default();
    let copied = unsafe {
        GetObjectW(
            HGDIOBJ(bitmap.0),
            size_of::<BITMAP>() as i32,
            Some(&mut info as *mut BITMAP as *mut c_void),
        )
    };
    if copied == 0 || info.bmWidth <= 0 || info.bmHeight <= 0 {
        return Err("Bitmap d'icône invalide".to_string());
    }

    let (width, height) = (info.bmWidth as usize, info.bmHeight as usize);
    let mut header = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            // Hauteur négative : lignes du haut vers le bas, sans retournement.
            biHeight: -(height as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut pixels = vec![0u8; width * height * 4];
    let device = unsafe { GetDC(None) };
    let lines = unsafe {
        GetDIBits(
            device,
            bitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut header,
            DIB_RGB_COLORS,
        )
    };
    unsafe { ReleaseDC(None, device) };
    if lines == 0 {
        return Err("Lecture des pixels de l'icône impossible".to_string());
    }

    let any_alpha = pixels.chunks_exact(4).any(|pixel| pixel[3] != 0);
    for pixel in pixels.chunks_exact_mut(4) {
        let (blue, green, red, alpha) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        if !any_alpha {
            pixel.copy_from_slice(&[red, green, blue, 255]);
        } else if alpha == 0 {
            pixel.copy_from_slice(&[0, 0, 0, 0]);
        } else {
            let unmultiply = |channel: u8| ((u32::from(channel) * 255) / u32::from(alpha)).min(255) as u8;
            pixel.copy_from_slice(&[unmultiply(red), unmultiply(green), unmultiply(blue), alpha]);
        }
    }

    RgbaImage::from_raw(width as u32, height as u32, pixels).ok_or_else(|| "Image d'icône invalide".to_string())
}
