//! Raccourcis du menu Démarrer, pour tous les utilisateurs et pour l'utilisateur courant.

use std::fs;
use std::path::{Path, PathBuf};

use super::model::{EntryKind, LauncherEntry};

const MAX_DEPTH: usize = 6;
/// Noms de raccourcis sans intérêt pour un lanceur.
const IGNORED_FRAGMENTS: &[&str] = &["uninstall", "désinstall", "desinstall"];

fn roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for variable in ["ProgramData", "APPDATA"] {
        if let Some(base) = std::env::var_os(variable) {
            roots.push(PathBuf::from(base).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
    }
    roots
}

fn walk(directory: &Path, depth: usize, entries: &mut Vec<LauncherEntry>) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(read) = fs::read_dir(directory) else {
        return;
    };

    for item in read.flatten() {
        let path = item.path();
        if path.is_dir() {
            walk(&path, depth + 1, entries);
            continue;
        }

        let is_shortcut = path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("lnk"));
        if !is_shortcut {
            continue;
        }

        let Some(name) = path.file_stem().map(|stem| stem.to_string_lossy().into_owned()) else {
            continue;
        };
        let lower = name.to_lowercase();
        if IGNORED_FRAGMENTS.iter().any(|fragment| lower.contains(fragment)) {
            continue;
        }

        entries.push(LauncherEntry {
            name,
            path: path.to_string_lossy().into_owned(),
            kind: EntryKind::App,
        });
    }
}

pub fn scan() -> Vec<LauncherEntry> {
    let mut entries = Vec::new();
    for root in roots() {
        walk(&root, 0, &mut entries);
    }
    entries
}
