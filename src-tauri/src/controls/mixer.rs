//! Mixeur par application, commun à tous les systèmes. Une application ouvre souvent plusieurs flux
//! audio (Chrome, Discord) : ils sont regroupés en une seule ligne, avec un seul curseur.

use super::types::AppVolume;

/// Clé des sons système de Windows (bips, notifications), qui n'appartiennent à aucune application.
pub const SYSTEM_SOUNDS_KEY: &str = "system";

/// Un flux audio tel que le système le décrit.
#[derive(Clone, Debug, PartialEq)]
pub struct Stream {
    /// Identifie l'application : nom de l'exécutable en minuscules, ou `system`.
    pub key: String,
    pub name: String,
    pub volume: u8,
    pub muted: bool,
    /// Fichier dont on tire l'icône : exécutable sous Windows, image du thème d'icônes sous Linux.
    pub icon_path: Option<String>,
}

/// Regroupe les flux par application. Le volume affiché est le plus fort des flux, et l'application
/// n'est muette que si tous ses flux le sont. Tri alphabétique, sons système en dernier.
pub fn group(streams: Vec<Stream>) -> Vec<AppVolume> {
    let mut apps: Vec<AppVolume> = Vec::new();
    for stream in streams {
        match apps.iter_mut().find(|app| app.id == stream.key) {
            Some(app) => {
                app.volume = app.volume.max(stream.volume);
                app.muted &= stream.muted;
                if app.icon_path.is_none() {
                    app.icon_path = stream.icon_path;
                }
            }
            None => apps.push(AppVolume {
                id: stream.key,
                name: stream.name,
                volume: stream.volume,
                muted: stream.muted,
                icon_path: stream.icon_path,
            }),
        }
    }
    apps.sort_by(|a, b| {
        (a.id == SYSTEM_SOUNDS_KEY)
            .cmp(&(b.id == SYSTEM_SOUNDS_KEY))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    apps
}

/// Nom lisible tiré d'un exécutable : `spotify.exe` donne `Spotify`.
pub fn display_name(executable: &str) -> String {
    let stem = executable.rsplit(['\\', '/']).next().unwrap_or(executable);
    let stem = stem.strip_suffix(".exe").or_else(|| stem.strip_suffix(".EXE")).unwrap_or(stem);
    let mut chars = stem.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(key: &str, name: &str, volume: u8, muted: bool) -> Stream {
        Stream { key: key.into(), name: name.into(), volume, muted, icon_path: None }
    }

    #[test]
    fn streams_of_one_app_become_one_line() {
        let apps = group(vec![
            stream("chrome.exe", "Chrome", 40, false),
            stream("spotify.exe", "Spotify", 70, false),
            stream("chrome.exe", "Chrome", 85, true),
        ]);
        assert_eq!(apps.len(), 2);
        let chrome = &apps[0];
        assert_eq!((chrome.id.as_str(), chrome.volume, chrome.muted), ("chrome.exe", 85, false));
    }

    #[test]
    fn an_app_is_muted_only_when_every_stream_is() {
        let apps = group(vec![stream("discord.exe", "Discord", 50, true), stream("discord.exe", "Discord", 50, true)]);
        assert!(apps[0].muted);
    }

    #[test]
    fn apps_are_sorted_by_name_with_system_sounds_last() {
        let apps = group(vec![
            stream(SYSTEM_SOUNDS_KEY, "Sons système", 100, false),
            stream("vlc.exe", "VLC", 60, false),
            stream("discord.exe", "discord", 50, false),
        ]);
        let names: Vec<&str> = apps.iter().map(|app| app.name.as_str()).collect();
        assert_eq!(names, vec!["discord", "VLC", "Sons système"]);
    }

    #[test]
    fn executable_names_are_made_readable() {
        assert_eq!(display_name(r"C:\Program Files\Spotify\Spotify.exe"), "Spotify");
        assert_eq!(display_name("firefox"), "Firefox");
        assert_eq!(display_name(""), "");
    }
}
