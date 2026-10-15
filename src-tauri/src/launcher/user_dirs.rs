//! Fichiers et dossiers des répertoires clés de l'utilisateur.

use std::fs;
use std::path::{Path, PathBuf};

use super::model::{EntryKind, LauncherEntry};

const MAX_DEPTH: usize = 3;
/// Plafond global pour garder l'index léger en mémoire.
const MAX_ENTRIES: usize = 30_000;
/// Dossiers volumineux et sans intérêt pour une recherche.
const SKIPPED_DIRECTORIES: &[&str] = &["node_modules", "target", ".git", "dist", "build", "__pycache__"];

fn walk(directory: &Path, depth: usize, entries: &mut Vec<LauncherEntry>) {
    if depth > MAX_DEPTH || entries.len() >= MAX_ENTRIES {
        return;
    }
    let Ok(read) = fs::read_dir(directory) else {
        return;
    };

    for item in read.flatten() {
        if entries.len() >= MAX_ENTRIES {
            return;
        }
        let path = item.path();
        let Some(name) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }

        let is_directory = item.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if is_directory && SKIPPED_DIRECTORIES.contains(&name.as_str()) {
            continue;
        }

        entries.push(LauncherEntry {
            name,
            path: path.to_string_lossy().into_owned(),
            kind: if is_directory { EntryKind::Folder } else { EntryKind::File },
        });

        if is_directory {
            walk(&path, depth + 1, entries);
        }
    }
}

pub fn scan(roots: &[PathBuf]) -> Vec<LauncherEntry> {
    let mut entries = Vec::new();
    for root in roots {
        walk(root, 0, &mut entries);
    }
    entries
}
