use std::path::PathBuf;

/// Dossiers de données de l'application, sous `%LocalAppData%\Katadacho`.
pub struct AppPaths {
    pub data_dir: PathBuf,
}

impl AppPaths {
    pub fn clipboard_images(&self) -> PathBuf {
        self.data_dir.join("clipboard_images")
    }

    pub fn icon_cache(&self) -> PathBuf {
        self.data_dir.join("icons")
    }
}
