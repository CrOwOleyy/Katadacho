pub mod migrations;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

/// Connexion SQLite partagée par toute l'application, protégée par un Mutex.
pub struct Database(pub Mutex<Connection>);

impl Database {
    /// Ouvre (ou crée) `katadacho.db` dans `dir`, en mode WAL, et applique les migrations.
    pub fn open(dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

        let connection = Connection::open(dir.join("katadacho.db")).map_err(|e| e.to_string())?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        connection
            .pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| e.to_string())?;
        migrations::run(&connection).map_err(|e| e.to_string())?;

        Ok(Self(Mutex::new(connection)))
    }

    /// Exécute `action` avec la connexion verrouillée. Un Mutex empoisonné est récupéré,
    /// car une panique dans un autre thread ne rend pas la base invalide.
    pub fn with<T>(&self, action: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Result<T, String> {
        let guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        action(&guard).map_err(|e| e.to_string())
    }
}
