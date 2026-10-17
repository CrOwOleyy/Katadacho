//! Notifications sous Linux. Toute application qui notifie appelle `Notify` sur le service
//! `org.freedesktop.Notifications` du bus de session. Katadacho écoute ces appels en mode moniteur
//! (comme `dbus-monitor`) : il les lit sans les intercepter, le bureau affiche donc aussi la sienne.
//! Aucune autorisation n'est à demander.

use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter, Manager};
use zbus::blocking::{Connection, MessageIterator, Proxy};
use zbus::message::Type as MessageType;
use zbus::zvariant::OwnedValue;

use super::{is_blocked, Toast, EVENT_NOTIFICATION};
use crate::settings::SettingsState;

const RETRY_DELAY: Duration = Duration::from_secs(5);
const MATCH_RULE: &str = "type='method_call',interface='org.freedesktop.Notifications',member='Notify'";

/// Corps de `Notify` : application, remplacement, icône, résumé, corps, actions, indices, durée.
type NotifyBody = (String, u32, String, String, String, Vec<String>, HashMap<String, OwnedValue>, i32);


/// Retire le balisage permis par la norme (`<b>`, `<i>`, `<a href>`…) et décode les entités courantes.
pub fn plain_text(markup: &str) -> String {
    let mut text = String::with_capacity(markup.len());
    let mut in_tag = false;
    for c in markup.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if in_tag => {}
            other => text.push(other),
        }
    }
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
        .trim()
        .to_string()
}

fn app_name(name: &str, hints: &HashMap<String, OwnedValue>) -> String {
    if !name.trim().is_empty() {
        return name.trim().to_string();
    }
    hints
        .get("desktop-entry")
        .and_then(|value| String::try_from(value.try_clone().ok()?).ok())
        .filter(|entry| !entry.is_empty())
        .unwrap_or_else(|| "Application".to_string())
}

fn to_toast(id: u32, body: NotifyBody) -> Option<Toast> {
    let (name, _replaces, _icon, summary, text, _actions, hints, _timeout) = body;
    let title = plain_text(&summary);
    let body = plain_text(&text);
    if title.is_empty() && body.is_empty() {
        return None;
    }
    // Sans résumé, le corps fait office de titre.
    let (title, body) = if title.is_empty() { (body, String::new()) } else { (title, body) };
    let created_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
    Some(Toast { id, app_name: app_name(&name, &hints), title, body, created_ms })
}

fn monitor(app: &AppHandle, next_id: &mut u32) -> Result<(), String> {
    let connection = Connection::session().map_err(|e| e.to_string())?;
    let bus = Proxy::new(&connection, "org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus.Monitoring")
        .map_err(|e| e.to_string())?;
    bus.call_method("BecomeMonitor", &(vec![MATCH_RULE], 0u32)).map_err(|e| e.to_string())?;

    for message in MessageIterator::from(connection) {
        let message = message.map_err(|e| e.to_string())?;
        if message.message_type() != MessageType::MethodCall {
            continue;
        }
        let Ok(body) = message.body().deserialize::<NotifyBody>() else {
            continue;
        };

        let settings = app.state::<SettingsState>().snapshot().notifications;
        if !settings.enabled {
            continue;
        }
        *next_id = next_id.wrapping_add(1);
        let Some(toast) = to_toast(*next_id, body) else {
            continue;
        };
        if is_blocked(&settings, &toast) {
            continue;
        }
        if settings.sound_enabled {
            crate::sound::play_notification();
        }
        let _ = app.emit(EVENT_NOTIFICATION, toast);
    }
    Err("Le bus de session s'est fermé".to_string())
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut next_id = 0u32;
        loop {
            if let Err(error) = monitor(&app, &mut next_id) {
                eprintln!("notification monitor stopped: {error}");
            }
            thread::sleep(RETRY_DELAY);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markup_is_removed_and_entities_decoded() {
        assert_eq!(plain_text("<b>Salut</b> <a href=\"x\">toi</a> &amp; moi"), "Salut toi & moi");
        assert_eq!(plain_text("  a &lt; b  "), "a < b");
    }

    #[test]
    fn a_toast_without_summary_uses_the_body_as_title() {
        let body: NotifyBody =
            ("Signal".into(), 0, String::new(), String::new(), "Hello".into(), vec![], HashMap::new(), -1);
        let toast = to_toast(7, body).unwrap();
        assert_eq!((toast.title.as_str(), toast.body.as_str(), toast.app_name.as_str()), ("Hello", "", "Signal"));
    }

    #[test]
    fn empty_notifications_are_dropped() {
        let body: NotifyBody = (String::new(), 0, String::new(), " ".into(), String::new(), vec![], HashMap::new(), -1);
        assert!(to_toast(1, body).is_none());
    }
}
