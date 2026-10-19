//! Applications installées sous Linux, lues dans les fichiers `.desktop` (norme freedesktop, utilisée
//! par toutes les distributions et par Flatpak et Snap).
// Ce module ne sert qu'hors Windows, mais il est compilé partout pour que ses tests tournent partout.
#![cfg_attr(windows, allow(dead_code))]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::model::{EntryKind, LauncherEntry};

const MAX_DEPTH: usize = 3;

/// Champs utiles d'une entrée `[Desktop Entry]`.
#[derive(Debug, PartialEq)]
pub struct DesktopEntry {
    pub name: String,
}

/// Langues à essayer pour `Name[xx]`, de la plus précise à la plus générale : `fr_FR`, puis `fr`.
fn preferred_locales() -> Vec<String> {
    let raw = std::env::var("LC_MESSAGES")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    let base = raw.split(['.', '@']).next().unwrap_or_default().to_string();
    let mut locales = Vec::new();
    if !base.is_empty() && base != "C" && base != "POSIX" {
        if let Some((language, _)) = base.split_once('_') {
            locales.push(base.clone());
            locales.push(language.to_string());
        } else {
            locales.push(base);
        }
    }
    locales
}

/// Lit une entrée de bureau. Retourne `None` si ce n'est pas une application visible.
pub fn parse(content: &str, locales: &[String]) -> Option<DesktopEntry> {
    let mut in_main_group = false;
    let mut name: Option<String> = None;
    let mut localized: Vec<(usize, String)> = Vec::new();
    let mut is_application = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_main_group = line == "[Desktop Entry]";
            continue;
        }
        if !in_main_group {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());

        match key {
            "Type" => is_application = value == "Application",
            "NoDisplay" | "Hidden" if value.eq_ignore_ascii_case("true") => return None,
            "Name" => name = Some(value.to_string()),
            _ => {
                if let Some(locale) = key.strip_prefix("Name[").and_then(|rest| rest.strip_suffix(']')) {
                    if let Some(rank) = locales.iter().position(|wanted| wanted == locale) {
                        localized.push((rank, value.to_string()));
                    }
                }
            }
        }
    }

    if !is_application {
        return None;
    }
    localized.sort_by_key(|(rank, _)| *rank);
    let name = localized.into_iter().next().map(|(_, value)| value).or(name)?;
    (!name.is_empty()).then_some(DesktopEntry { name })
}

/// Dossiers `applications` dans l'ordre de priorité freedesktop : l'utilisateur d'abord.
fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    if let Some(data_home) = data_home {
        dirs.push(data_home.join("applications"));
        dirs.push(data_home.join("flatpak/exports/share/applications"));
    }

    let data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    dirs.extend(data_dirs.split(':').filter(|dir| !dir.is_empty()).map(|dir| Path::new(dir).join("applications")));

    // Emplacements de Flatpak et Snap, parfois absents de XDG_DATA_DIRS.
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

    let mut seen = HashSet::new();
    dirs.retain(|dir| seen.insert(dir.clone()));
    dirs
}

fn collect(dir: &Path, root: &Path, depth: usize, seen_ids: &mut HashSet<String>, locales: &[String], out: &mut Vec<LauncherEntry>) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for item in read.flatten() {
        let path = item.path();
        if path.is_dir() {
            collect(&path, root, depth + 1, seen_ids, locales, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("desktop") {
            continue;
        }

        // L'identifiant freedesktop est le chemin relatif, sous-dossiers séparés par « - ».
        let id = path
            .strip_prefix(root)
            .map(|relative| relative.to_string_lossy().replace(['/', '\\'], "-"))
            .unwrap_or_default();
        // Une entrée de l'utilisateur masque l'entrée système du même identifiant, même si elle est cachée.
        if !seen_ids.insert(id) {
            continue;
        }

        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        if let Some(entry) = parse(&content, locales) {
            out.push(LauncherEntry {
                name: entry.name,
                path: path.to_string_lossy().into_owned(),
                kind: EntryKind::App,
            });
        }
    }
}

pub fn scan() -> Vec<LauncherEntry> {
    let locales = preferred_locales();
    let mut seen_ids = HashSet::new();
    let mut entries = Vec::new();
    for dir in application_dirs() {
        collect(&dir, &dir, 0, &mut seen_ids, &locales, &mut entries);
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn french() -> Vec<String> {
        vec!["fr_FR".to_string(), "fr".to_string()]
    }

    #[test]
    fn reads_an_application_name() {
        let entry = parse("[Desktop Entry]\nType=Application\nName=Firefox\nExec=firefox %u\n", &[]);
        assert_eq!(entry, Some(DesktopEntry { name: "Firefox".into() }));
    }

    #[test]
    fn prefers_the_most_precise_translation() {
        let content = "[Desktop Entry]\nType=Application\nName=Files\nName[fr]=Fichiers\nName[fr_FR]=Fichiers (FR)\n";
        assert_eq!(parse(content, &french()).map(|e| e.name), Some("Fichiers (FR)".into()));
        let generic = "[Desktop Entry]\nType=Application\nName=Files\nName[fr]=Fichiers\n";
        assert_eq!(parse(generic, &french()).map(|e| e.name), Some("Fichiers".into()));
    }

    #[test]
    fn skips_hidden_entries_and_non_applications() {
        assert_eq!(parse("[Desktop Entry]\nType=Application\nName=X\nNoDisplay=true\n", &[]), None);
        assert_eq!(parse("[Desktop Entry]\nType=Application\nName=X\nHidden=true\n", &[]), None);
        assert_eq!(parse("[Desktop Entry]\nType=Link\nName=Site\nURL=https://example.com\n", &[]), None);
    }

    #[test]
    fn ignores_keys_from_action_groups() {
        let content = "[Desktop Entry]\nType=Application\nName=Firefox\n\n[Desktop Action new-window]\nName=Nouvelle fenêtre\n";
        assert_eq!(parse(content, &[]).map(|e| e.name), Some("Firefox".into()));
    }
}
