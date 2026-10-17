//! Lecture des notifications toast de Windows via `UserNotificationListener`.
//! L'utilisateur doit autoriser l'accès aux notifications pour l'application.

use super::types::{Access, Toast};
use windows::UI::Notifications::Management::{UserNotificationListener, UserNotificationListenerAccessStatus};
use windows::UI::Notifications::{KnownNotificationBindings, NotificationKinds, UserNotification};

/// Écart entre l'époque Windows (1601) et l'époque Unix (1970), en millisecondes.
const WINDOWS_TO_UNIX_EPOCH_MS: i64 = 11_644_473_600_000;
/// Un DateTime Windows compte des intervalles de 100 nanosecondes.
const TICKS_PER_MS: i64 = 10_000;

fn convert(status: UserNotificationListenerAccessStatus) -> Access {
    if status == UserNotificationListenerAccessStatus::Allowed {
        Access::Allowed
    } else if status == UserNotificationListenerAccessStatus::Denied {
        Access::Denied
    } else {
        Access::Unspecified
    }
}

pub fn access_status() -> Result<Access, String> {
    let listener = UserNotificationListener::Current().map_err(|e| e.to_string())?;
    listener.GetAccessStatus().map(convert).map_err(|e| e.to_string())
}

/// Demande l'autorisation. Windows n'affiche la fenêtre de consentement qu'une seule fois.
pub fn request_access() -> Result<Access, String> {
    let listener = UserNotificationListener::Current().map_err(|e| e.to_string())?;
    listener
        .RequestAccessAsync()
        .and_then(|operation| operation.join())
        .map(convert)
        .map_err(|e| e.to_string())
}

fn to_toast(notification: &UserNotification) -> Option<Toast> {
    let id = notification.Id().ok()?;
    let app_name = notification
        .AppInfo()
        .and_then(|info| info.DisplayInfo())
        .and_then(|display| display.DisplayName())
        .map(|name| name.to_string())
        .unwrap_or_else(|_| "Application".to_string());

    let binding = notification
        .Notification()
        .and_then(|content| content.Visual())
        .and_then(|visual| KnownNotificationBindings::ToastGeneric().and_then(|name| visual.GetBinding(&name)))
        .ok()?;

    let elements = binding.GetTextElements().ok()?;
    let mut texts: Vec<String> = Vec::new();
    for index in 0..elements.Size().ok()? {
        if let Ok(text) = elements.GetAt(index).and_then(|element| element.Text()) {
            let text = text.to_string();
            if !text.trim().is_empty() {
                texts.push(text);
            }
        }
    }
    if texts.is_empty() {
        return None;
    }

    let title = texts.remove(0);
    let created_ms = notification
        .CreationTime()
        .map(|time| time.UniversalTime / TICKS_PER_MS - WINDOWS_TO_UNIX_EPOCH_MS)
        .unwrap_or(0);

    Some(Toast { id, app_name, title, body: texts.join("\n"), created_ms })
}

/// Toasts actuellement présents dans le centre de notifications. Le COM doit être initialisé.
pub fn fetch_toasts() -> Result<Vec<Toast>, String> {
    let listener = UserNotificationListener::Current().map_err(|e| e.to_string())?;
    let list = listener
        .GetNotificationsAsync(NotificationKinds::Toast)
        .and_then(|operation| operation.join())
        .map_err(|e| e.to_string())?;

    let mut toasts = Vec::new();
    for index in 0..list.Size().map_err(|e| e.to_string())? {
        if let Some(toast) = list.GetAt(index).ok().and_then(|n| to_toast(&n)) {
            toasts.push(toast);
        }
    }
    Ok(toasts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::ComGuard;

    /// Test manuel : poste d'abord un toast (par exemple avec PowerShell), puis lance
    /// `cargo test --lib notifications -- --ignored --nocapture`.
    #[test]
    #[ignore = "dépend d'une notification réelle présente dans Windows"]
    fn lists_real_toasts() {
        let _com = ComGuard::new().expect("COM");
        let access = access_status().expect("statut d'accès");
        println!("accès : {access:?}");
        assert_eq!(access, Access::Allowed);

        let toasts = fetch_toasts().expect("lecture des toasts");
        for toast in &toasts {
            println!("{} | {} | {} | {}", toast.id, toast.app_name, toast.title, toast.body);
        }
        assert!(!toasts.is_empty(), "aucun toast trouvé : poste-en un avant le test");
    }
}
