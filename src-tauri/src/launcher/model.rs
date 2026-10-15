use serde::Serialize;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    /// Raccourci du menu Démarrer vers une application de bureau.
    App,
    /// Application du Microsoft Store.
    Uwp,
    File,
    Folder,
}

/// Élément indexé en mémoire et cherchable par nom.
#[derive(Clone, Debug)]
pub struct LauncherEntry {
    pub name: String,
    /// Chemin à lancer. Pour une application UWP : `shell:AppsFolder\<AppUserModelId>`.
    pub path: String,
    pub kind: EntryKind,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LauncherResult {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub score: u32,
}
