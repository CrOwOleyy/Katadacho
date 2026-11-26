//! Recherche d'une icône d'application dans les thèmes d'icônes Linux (norme freedesktop) : un nom
//! comme `firefox` devient un fichier PNG. Seul le thème de repli `hicolor`, que toutes les
//! applications remplissent, est parcouru.
// Sous Windows les icônes viennent des exécutables : ce module ne sert qu'aux tests.
#![cfg_attr(windows, allow(dead_code))]

use std::path::{Path, PathBuf};

/// Tailles parcourues, de la plus nette à la plus petite.
const SIZES: &[&str] = &["256x256", "128x128", "96x96", "64x64", "48x48", "32x32"];
pub const PIXMAPS_DIR: &str = "/usr/share/pixmaps";

/// Dossiers de thèmes, du plus personnel au plus général, Flatpak compris.
pub fn icon_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        roots.push(home.join(".local/share/icons"));
        roots.push(home.join(".local/share/flatpak/exports/share/icons"));
        roots.push(home.join(".icons"));
    }
    let data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    roots.extend(data_dirs.split(':').filter(|dir| !dir.is_empty()).map(|dir| Path::new(dir).join("icons")));
    roots.push(PathBuf::from("/var/lib/flatpak/exports/share/icons"));
    roots
}

/// PNG d'une icône nommée, ou `None` quand elle n'existe qu'en SVG ou pas du tout.
/// Un chemin absolu vers un PNG est accepté tel quel.
pub fn find_png(name: &str, roots: &[PathBuf], pixmaps: &Path) -> Option<PathBuf> {
    let candidate = Path::new(name);
    if candidate.is_absolute() {
        let is_png = candidate.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("png"));
        return (is_png && candidate.is_file()).then(|| candidate.to_path_buf());
    }
    let file = format!("{name}.png");
    let file = file.as_str();
    roots
        .iter()
        .flat_map(|root| SIZES.iter().map(move |size| root.join("hicolor").join(size).join("apps").join(file)))
        .chain(std::iter::once(pixmaps.join(file)))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Faux thème d'icônes dans un dossier temporaire propre au test.
    fn theme(test: &str, files: &[&str]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("katadacho-icon-theme-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for file in files {
            let path = root.join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"png").unwrap();
        }
        root
    }

    #[test]
    fn the_sharpest_hicolor_png_wins() {
        let root = theme("sharpest", &["hicolor/48x48/apps/firefox.png", "hicolor/256x256/apps/firefox.png"]);
        let found = find_png("firefox", &[root.clone()], &root.join("pixmaps"));
        assert_eq!(found, Some(root.join("hicolor/256x256/apps/firefox.png")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pixmaps_are_the_last_resort_and_svg_only_icons_are_skipped() {
        let root = theme("fallback", &["pixmaps/vlc.png", "hicolor/scalable/apps/spotify.svg"]);
        assert_eq!(find_png("vlc", &[root.clone()], &root.join("pixmaps")), Some(root.join("pixmaps/vlc.png")));
        assert_eq!(find_png("spotify", &[root.clone()], &root.join("pixmaps")), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_absolute_png_path_is_used_as_is() {
        let root = theme("absolute", &["mon-icone.png"]);
        let path = root.join("mon-icone.png");
        let found = find_png(path.to_str().unwrap(), &[], &root);
        assert_eq!(found, Some(path));
        fs::remove_dir_all(root).unwrap();
    }
}
