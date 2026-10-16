use std::fs;
use std::path::Path;

use tauri::State;

use super::store::{self, ClipboardItem};
use super::{attach_thumbnail, thumbnail_path};
use crate::db::Database;

const DEFAULT_LIMIT: u32 = 200;
const MAX_LIMIT: u32 = 2000;

/// Historique du presse-papier. `filter` : `all`, `text`, `image`, `link` ou `pinned`.
#[tauri::command]
pub async fn get_clipboard_history(
    database: State<'_, Database>,
    search_query: Option<String>,
    filter: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<ClipboardItem>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let mut items = store::history(
        &database,
        search_query.as_deref(),
        filter.as_deref().unwrap_or("all"),
        limit,
    )?;
    items.iter_mut().for_each(attach_thumbnail);
    Ok(items)
}

#[tauri::command]
pub async fn toggle_pin_clipboard(database: State<'_, Database>, id: i64) -> Result<bool, String> {
    store::toggle_pin(&database, id)
}

/// Supprime l'élément et, pour une image, ses fichiers sur le disque.
#[tauri::command]
pub async fn delete_clipboard_item(database: State<'_, Database>, id: i64) -> Result<(), String> {
    if let Some(image_path) = store::delete(&database, id)? {
        let full = Path::new(&image_path);
        let _ = fs::remove_file(full);
        let _ = fs::remove_file(thumbnail_path(full));
    }
    Ok(())
}

/// Remet l'élément dans le presse-papier Windows. Avec `simulate_paste`, rend aussi le focus
/// à la fenêtre précédemment active et simule `Ctrl+V`.
#[tauri::command]
pub async fn paste_clipboard_item(
    database: State<'_, Database>,
    id: i64,
    simulate_paste: Option<bool>,
) -> Result<(), String> {
    let item = store::get(&database, id)?.ok_or_else(|| format!("Élément {id} introuvable"))?;

    tauri::async_runtime::spawn_blocking(move || put_back(&item, simulate_paste.unwrap_or(false)))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(windows)]
fn put_back(item: &ClipboardItem, simulate_paste: bool) -> Result<(), String> {
    use super::{foreground, paste};

    match (&item.text_content, &item.image_path) {
        (Some(text), _) => paste::put_text(text)?,
        (None, Some(path)) => paste::put_image(Path::new(path))?,
        (None, None) => return Err("Élément vide".to_string()),
    }

    if simulate_paste {
        let target = foreground::last_external_window()
            .ok_or_else(|| "Aucune fenêtre précédente où coller".to_string())?;
        paste::paste_into(target)?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn put_back(_item: &ClipboardItem, _simulate_paste: bool) -> Result<(), String> {
    Err(crate::platform::unsupported("Le collage depuis l'historique"))
}
