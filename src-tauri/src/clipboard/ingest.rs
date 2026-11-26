//! Enregistrement d'une copie, identique sur tous les systèmes : hash, sauvegarde des images,
//! enregistrement en base, puis notification du frontend.

use std::fs;
use std::path::{Path, PathBuf};

use image::RgbaImage;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};

use super::classify::{classify_text, ContentKind};
use super::store::{self, NewItem};
use super::{attach_thumbnail, secrets, thumbnail_path, EVENT_CLIPBOARD_UPDATED};
use crate::db::Database;
use crate::settings::SettingsState;
use crate::state::AppPaths;

const THUMBNAIL_MAX_WIDTH: u32 = 320;
const THUMBNAIL_MAX_HEIGHT: u32 = 200;

/// Ce qui a été copié, lu par le système de capture.
pub enum ClipboardContent {
    Text(String),
    Image(RgbaImage),
}


fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
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

/// Classe, déduplique et enregistre une copie, puis notifie le frontend.
pub fn ingest(app: &AppHandle, content: ClipboardContent, source: Option<String>) {
    let database = app.state::<Database>();
    let paths = app.state::<AppPaths>();

    let result = match &content {
        ClipboardContent::Text(text) => {
            if text.trim().is_empty() {
                return;
            }
            let kind = classify_text(text);
            let clipboard_settings = app.state::<SettingsState>().snapshot().clipboard;
            let is_sensitive = clipboard_settings.detect_secrets && secrets::looks_secret(text);
            store::insert_or_bump(
                &database,
                &NewItem {
                    content_type: kind.as_str(),
                    text_content: Some(text),
                    image_path: None,
                    source_app_name: source.as_deref(),
                    content_hash: &hash_bytes(text.as_bytes()),
                    is_sensitive,
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
                    is_sensitive: false,
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
