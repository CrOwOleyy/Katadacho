pub mod commands;
pub mod hotkey;
pub mod icons;
pub mod model;
pub mod search;
pub mod start_menu;
pub mod user_dirs;
pub mod uwp;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use std::thread;

use nucleo_matcher::{Config, Matcher};
use tauri::{AppHandle, Manager};

use crate::controls::audio::ComGuard;
use model::{EntryKind, LauncherEntry, LauncherResult};

const RESULT_LIMIT: usize = 30;

/// Index en mémoire des applications, fichiers et dossiers cherchables.
pub struct Launcher {
    entries: RwLock<Vec<LauncherEntry>>,
    matcher: Mutex<Matcher>,
}

impl Default for Launcher {
    fn default() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            matcher: Mutex::new(Matcher::new(Config::DEFAULT)),
        }
    }
}

impl Launcher {
    /// Ajoute des entrées en évitant les doublons : un même nom d'application n'apparaît
    /// qu'une fois, un fichier est identifié par son chemin.
    fn add(&self, new_entries: Vec<LauncherEntry>) {
        let mut guard = self.entries.write().unwrap_or_else(|e| e.into_inner());

        let key = |entry: &LauncherEntry| match entry.kind {
            EntryKind::App | EntryKind::Uwp => format!("app:{}", entry.name.to_lowercase()),
            EntryKind::File | EntryKind::Folder => format!("path:{}", entry.path.to_lowercase()),
        };
        let mut seen: HashSet<String> = guard.iter().map(key).collect();

        for entry in new_entries {
            if seen.insert(key(&entry)) {
                guard.push(entry);
            }
        }
    }

    fn clear(&self) {
        self.entries.write().unwrap_or_else(|e| e.into_inner()).clear();
    }

    pub fn search(&self, query: &str) -> Vec<LauncherResult> {
        let entries = self.entries.read().unwrap_or_else(|e| e.into_inner());
        let mut matcher = self.matcher.lock().unwrap_or_else(|e| e.into_inner());
        search::search(&entries, &mut matcher, query, RESULT_LIMIT)
    }
}

fn user_roots(app: &AppHandle) -> Vec<PathBuf> {
    let resolver = app.path();
    let mut roots: Vec<PathBuf> = [resolver.desktop_dir(), resolver.download_dir(), resolver.document_dir()]
        .into_iter()
        .flatten()
        .collect();

    if let Ok(home) = resolver.home_dir() {
        let projects = home.join("Projects");
        if projects.is_dir() {
            roots.push(projects);
        }
    }
    roots
}

/// Construit l'index en arrière-plan, du plus rapide au plus lent, pour que la recherche
/// d'applications soit utilisable immédiatement.
pub fn start_indexing(app: AppHandle) {
    thread::spawn(move || {
        let launcher = app.state::<Launcher>();

        launcher.add(start_menu::scan());

        if let Ok(_com) = ComGuard::new() {
            launcher.add(uwp::scan());
        }

        launcher.add(user_dirs::scan(&user_roots(&app)));
    });
}

/// Vide l'index et le reconstruit, par exemple après l'installation d'une application.
pub fn reindex(app: AppHandle) {
    app.state::<Launcher>().clear();
    start_indexing(app);
}
