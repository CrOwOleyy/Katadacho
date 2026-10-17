//! Presse-papier sous Linux (X11 et Wayland) via `arboard`. Il n'existe pas de notification de copie
//! commune à tous les bureaux : le contenu est relu quatre fois par seconde et seul un changement
//! est enregistré. Les copies marquées par un gestionnaire de mots de passe sont ignorées.

use std::borrow::Cow;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use arboard::{Clipboard, ImageData};
use image::RgbaImage;
use sha2::{Digest, Sha256};
use tauri::AppHandle;

use super::ingest::{self, ClipboardContent};

const POLL_INTERVAL: Duration = Duration::from_millis(250);
/// Délai laissé à la fenêtre cible pour reprendre le focus avant d'envoyer Ctrl+V.
const PASTE_DELAY: Duration = Duration::from_millis(150);
/// Marqueur posé par KeePassXC, Bitwarden, 1Password et d'autres sur leurs copies.
const PASSWORD_MANAGER_HINT: &str = "x-kde-passwordManagerHint";

/// Sous X11, le contenu copié par l'application disparaît avec l'objet `Clipboard` : un seul exemplaire
/// partagé reste en vie pendant toute l'exécution.
static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

fn with_clipboard<T>(action: impl FnOnce(&mut Clipboard) -> Result<T, arboard::Error>) -> Result<T, String> {
    let mut guard = CLIPBOARD.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_none() {
        *guard = Some(Clipboard::new().map_err(|e| format!("Presse-papier indisponible : {e}"))?);
    }
    let result = action(guard.as_mut().expect("initialisé ci-dessus"));
    if result.is_err() {
        // Une connexion cassée (serveur d'affichage relancé) est recréée à la prochaine lecture.
        let broken = matches!(&result, Err(arboard::Error::Unknown { .. }));
        if broken {
            *guard = None;
        }
    }
    result.map_err(|e| e.to_string())
}

/// `true` quand la liste de types MIME du presse-papier contient le marqueur de gestionnaire de mots de passe.
pub fn has_password_manager_hint(types: &str) -> bool {
    types.lines().any(|line| line.trim() == PASSWORD_MANAGER_HINT)
}

fn clipboard_types() -> Option<String> {
    let output = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        Command::new("wl-paste").arg("--list-types").stderr(Stdio::null()).output().ok()?
    } else {
        Command::new("xclip")
            .args(["-selection", "clipboard", "-o", "-t", "TARGETS"])
            .stderr(Stdio::null())
            .output()
            .ok()?
    };
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn read_content() -> Option<ClipboardContent> {
    with_clipboard(|clipboard| match clipboard.get_text() {
        Ok(text) if !text.trim().is_empty() => Ok(Some(ClipboardContent::Text(text))),
        _ => match clipboard.get_image() {
            Ok(image) => {
                Ok(RgbaImage::from_raw(image.width as u32, image.height as u32, image.bytes.into_owned())
                    .map(ClipboardContent::Image))
            }
            Err(error) => Err(error),
        },
    })
    .ok()
    .flatten()
}

fn fingerprint(content: &ClipboardContent) -> String {
    let digest = match content {
        ClipboardContent::Text(text) => Sha256::digest(text.as_bytes()),
        ClipboardContent::Image(image) => Sha256::digest(image.as_raw()),
    };
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Lance la lecture régulière du presse-papier.
pub fn start(app: AppHandle) {
    thread::spawn(move || {
        // Ce qui est déjà copié au démarrage n'est pas une nouvelle copie.
        let mut last = read_content().map(|content| fingerprint(&content));

        loop {
            thread::sleep(POLL_INTERVAL);
            let Some(content) = read_content() else {
                continue;
            };
            let current = fingerprint(&content);
            if last.as_deref() == Some(current.as_str()) {
                continue;
            }
            last = Some(current);

            if clipboard_types().is_some_and(|types| has_password_manager_hint(&types)) {
                continue;
            }
            ingest::ingest(&app, content, None);
        }
    });
}

fn send_paste_shortcut() -> Result<(), String> {
    thread::sleep(PASTE_DELAY);
    let attempt = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        Command::new("wtype").args(["-M", "ctrl", "v", "-m", "ctrl"]).status()
    } else {
        Command::new("xdotool").args(["key", "--clearmodifiers", "ctrl+v"]).status()
    };
    match attempt {
        Ok(status) if status.success() => Ok(()),
        _ => Err("Copié, mais le collage automatique demande xdotool (X11) ou wtype (Wayland)".to_string()),
    }
}

/// Remet l'élément dans le presse-papier, puis envoie Ctrl+V si demandé.
pub fn put_back(text: Option<&str>, image_path: Option<&Path>, simulate_paste: bool) -> Result<(), String> {
    match (text, image_path) {
        (Some(text), _) => with_clipboard(|clipboard| clipboard.set_text(text))?,
        (None, Some(path)) => {
            let image = image::open(path).map_err(|e| e.to_string())?.to_rgba8();
            let (width, height) = image.dimensions();
            with_clipboard(|clipboard| {
                clipboard.set_image(ImageData {
                    width: width as usize,
                    height: height as usize,
                    bytes: Cow::Borrowed(image.as_raw()),
                })
            })?;
        }
        (None, None) => return Err("Élément vide".to_string()),
    }
    if simulate_paste {
        send_paste_shortcut()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_password_manager_marker_is_recognised() {
        assert!(has_password_manager_hint("text/plain\nx-kde-passwordManagerHint\nUTF8_STRING\n"));
        assert!(!has_password_manager_hint("text/plain\nUTF8_STRING\n"));
    }

    #[test]
    fn identical_content_has_the_same_fingerprint() {
        let a = ClipboardContent::Text("bonjour".into());
        let b = ClipboardContent::Text("bonjour".into());
        assert_eq!(fingerprint(&a), fingerprint(&b));
        assert_ne!(fingerprint(&a), fingerprint(&ClipboardContent::Text("autre".into())));
    }
}
