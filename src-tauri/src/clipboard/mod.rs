pub mod classify;
pub mod listener;
pub mod reader;
pub mod source;
pub mod store;

use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::RgbaImage;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::HWND;

use crate::db::Database;
use crate::state::AppPaths;
use classify::{classify_text, ContentKind};
use reader::ClipboardContent;
use store::{ClipboardItem, NewItem};

pub const EVENT_CLIPBOARD_UPDATED: &str = "clipboard-updated";
const THUMBNAIL_MAX_WIDTH: u32 = 320;
const THUMBNAIL_MAX_HEIGHT: u32 = 200;

fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Chemin de la miniature d'une image : même nom avec le suffixe `_thumb`.
pub fn thumbnail_path(image_path: &Path) -> PathBuf {
    let stem = image_path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    image_path.with_file_name(format!("{stem}_thumb.png"))
}

/// Ajoute la miniature en base64 aux éléments image, lue depuis le disque.
pub fn attach_thumbnail(item: &mut ClipboardItem) {
    let Some(path) = item.image_path.as_deref() else {
        return;
    };
    if let Ok(bytes) = fs::read(thumbnail_path(Path::new(path))) {
        item.thumbnail_base64 = Some(STANDARD.encode(bytes));
    }
}

fn save_image(directory: &Path, hash: &str, image: &RgbaImage) -> Result<PathBuf, String> {
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;

    let full = directory.join(format!("{hash}.png"));
    if !full.exists() {
        image.save(&full).map_err(|e| e.to_string())?;
        let thumbnail = image::imageops::thumbnail(
            image,
            image.width().min(THUMBNAIL_MAX_WIDTH),
            image.height().min(THUMBNAIL_MAX_HEIGHT),
        );
        thumbnail.save(thumbnail_path(&full)).map_err(|e| e.to_string())?;
    }
    Ok(full)
}

/// Appelé par le listener à chaque copie : lit, classe, déduplique, enregistre, puis notifie le frontend.
pub fn capture(app: &AppHandle, owner: HWND) {
    let Some(content) = reader::read(Some(owner)) else {
        return;
    };
    let source = source::foreground_app_name();
    let database = app.state::<Database>();
    let paths = app.state::<AppPaths>();

    let result = match &content {
        ClipboardContent::Text(text) => {
            if text.trim().is_empty() {
                return;
            }
            let kind = classify_text(text);
            store::insert_or_bump(
                &database,
                &NewItem {
                    content_type: kind.as_str(),
                    text_content: Some(text),
                    image_path: None,
                    source_app_name: source.as_deref(),
                    content_hash: &hash_bytes(text.as_bytes()),
                },
            )
        }
        ClipboardContent::Image(image) => {
            let hash = hash_bytes(image.as_raw());
            let path = match save_image(&paths.clipboard_images(), &hash, image) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("clipboard image save failed: {error}");
                    return;
                }
            };
            store::insert_or_bump(
                &database,
                &NewItem {
                    content_type: ContentKind::Image.as_str(),
                    text_content: None,
                    image_path: Some(&path.to_string_lossy()),
                    source_app_name: source.as_deref(),
                    content_hash: &hash,
                },
            )
        }
    };

    match result {
        Ok(mut item) => {
            attach_thumbnail(&mut item);
            let _ = app.emit(EVENT_CLIPBOARD_UPDATED, item);
        }
        Err(error) => eprintln!("clipboard store failed: {error}"),
    }
}
