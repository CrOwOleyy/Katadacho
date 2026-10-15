use rusqlite::Connection;

/// Chaque entrée est appliquée une seule fois, dans l'ordre. L'index + 1 est la version
/// stockée dans `PRAGMA user_version`. On n'édite jamais une migration déjà publiée :
/// on en ajoute une nouvelle à la fin.
const MIGRATIONS: &[&str] = &[
    "CREATE TABLE clipboard_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        content_type TEXT NOT NULL,
        text_content TEXT,
        image_path TEXT,
        source_app_name TEXT,
        is_pinned INTEGER NOT NULL DEFAULT 0,
        created_at INTEGER NOT NULL,
        content_hash TEXT NOT NULL
    );
    CREATE INDEX idx_clipboard_created ON clipboard_items (created_at DESC);
    CREATE INDEX idx_clipboard_hash ON clipboard_items (content_hash);",
    "CREATE TABLE settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );",
];

pub fn run(connection: &Connection) -> rusqlite::Result<()> {
    let current: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let transaction = connection.unchecked_transaction()?;
        transaction.execute_batch(sql)?;
        transaction.pragma_update(None, "user_version", index as i64 + 1)?;
        transaction.commit()?;
    }
    Ok(())
}
