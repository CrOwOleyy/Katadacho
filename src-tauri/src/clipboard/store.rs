use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::Serialize;

use crate::db::Database;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: i64,
    pub content_type: String,
    pub text_content: Option<String>,
    pub image_path: Option<String>,
    /// Miniature PNG encodée en base64, remplie pour les images uniquement.
    pub thumbnail_base64: Option<String>,
    pub source_app_name: Option<String>,
    pub is_pinned: bool,
    /// Ressemble à un secret : affiché masqué et effacé automatiquement.
    pub is_sensitive: bool,
    pub created_at: i64,
}

pub struct NewItem<'a> {
    pub content_type: &'a str,
    pub text_content: Option<&'a str>,
    pub image_path: Option<&'a str>,
    pub source_app_name: Option<&'a str>,
    pub content_hash: &'a str,
    pub is_sensitive: bool,
}

const COLUMNS: &str =
    "id, content_type, text_content, image_path, source_app_name, is_pinned, created_at, is_sensitive";

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn row_to_item(row: &Row) -> rusqlite::Result<ClipboardItem> {
    Ok(ClipboardItem {
        id: row.get(0)?,
        content_type: row.get(1)?,
        text_content: row.get(2)?,
        image_path: row.get(3)?,
        thumbnail_base64: None,
        source_app_name: row.get(4)?,
        is_pinned: row.get::<_, i64>(5)? != 0,
        created_at: row.get(6)?,
        is_sensitive: row.get::<_, i64>(7)? != 0,
    })
}

fn get_with(conn: &Connection, id: i64) -> rusqlite::Result<Option<ClipboardItem>> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM clipboard_items WHERE id = ?1"),
        params![id],
        row_to_item,
    )
    .optional()
}

/// Insère un élément. Si le même contenu (même hash) existe déjà, il remonte en tête
/// de liste au lieu d'être dupliqué, et garde son état épinglé.
pub fn insert_or_bump(db: &Database, item: &NewItem) -> Result<ClipboardItem, String> {
    db.with(|conn| {
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM clipboard_items WHERE content_hash = ?1",
                params![item.content_hash],
                |row| row.get(0),
            )
            .optional()?;

        let id = match existing {
            Some(id) => {
                conn.execute(
                    "UPDATE clipboard_items SET created_at = ?1, source_app_name = COALESCE(?2, source_app_name), is_sensitive = ?4 WHERE id = ?3",
                    params![now_ms(), item.source_app_name, id, item.is_sensitive],
                )?;
                id
            }
            None => {
                conn.execute(
                    "INSERT INTO clipboard_items (content_type, text_content, image_path, source_app_name, is_pinned, created_at, content_hash, is_sensitive)
                     VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7)",
                    params![
                        item.content_type,
                        item.text_content,
                        item.image_path,
                        item.source_app_name,
                        now_ms(),
                        item.content_hash,
                        item.is_sensitive
                    ],
                )?;
                conn.last_insert_rowid()
            }
        };

        get_with(conn, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
    })
}

pub fn get(db: &Database, id: i64) -> Result<Option<ClipboardItem>, String> {
    db.with(|conn| get_with(conn, id))
}

/// Historique du plus récent au plus ancien, épinglés d'abord.
/// `filter` : `all`, `text`, `image`, `link` ou `pinned`.
pub fn history(
    db: &Database,
    search: Option<&str>,
    filter: &str,
    limit: u32,
) -> Result<Vec<ClipboardItem>, String> {
    let mut conditions: Vec<&str> = Vec::new();
    match filter {
        "text" => conditions.push("content_type IN ('text', 'code', 'color')"),
        "image" => conditions.push("content_type = 'image'"),
        "link" => conditions.push("content_type = 'url'"),
        "pinned" => conditions.push("is_pinned = 1"),
        _ => {}
    }

    let pattern = search
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")));
    if pattern.is_some() {
        conditions.push("(text_content LIKE ?1 ESCAPE '\\' OR source_app_name LIKE ?1 ESCAPE '\\')");
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    let sql = format!(
        "SELECT {COLUMNS} FROM clipboard_items {where_clause} ORDER BY is_pinned DESC, created_at DESC LIMIT {limit}"
    );

    db.with(|conn| {
        let mut statement = conn.prepare(&sql)?;
        let rows = match &pattern {
            Some(pattern) => statement.query_map(params![pattern], row_to_item)?,
            None => statement.query_map([], row_to_item)?,
        };
        rows.collect()
    })
}

/// Inverse l'état épinglé et retourne le nouvel état.
pub fn toggle_pin(db: &Database, id: i64) -> Result<bool, String> {
    db.with(|conn| {
        conn.execute(
            "UPDATE clipboard_items SET is_pinned = 1 - is_pinned WHERE id = ?1",
            params![id],
        )?;
        conn.query_row(
            "SELECT is_pinned FROM clipboard_items WHERE id = ?1",
            params![id],
            |row| Ok(row.get::<_, i64>(0)? != 0),
        )
    })
}

/// Supprime l'élément et retourne son chemin d'image, à effacer du disque par l'appelant.
pub fn delete(db: &Database, id: i64) -> Result<Option<String>, String> {
    db.with(|conn| {
        let image_path: Option<String> = conn
            .query_row(
                "SELECT image_path FROM clipboard_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?
            .flatten();
        conn.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
        Ok(image_path)
    })
}

/// Efface les secrets non épinglés copiés avant `cutoff_ms`. Retourne le nombre d'éléments effacés.
pub fn delete_expired_sensitive(db: &Database, cutoff_ms: i64) -> Result<usize, String> {
    db.with(|conn| {
        conn.execute(
            "DELETE FROM clipboard_items WHERE is_sensitive = 1 AND is_pinned = 0 AND created_at < ?1",
            params![cutoff_ms],
        )
    })
}
