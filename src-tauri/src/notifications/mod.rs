#[cfg(target_os = "linux")]
mod linux;
#[cfg(windows)]
pub mod listener;
mod types;

use tauri::AppHandle;

use crate::settings::NotificationSettings;
pub use types::{Access, Toast};

pub const EVENT_NOTIFICATION: &str = "notification-arrived";

fn is_blocked(settings: &NotificationSettings, toast: &Toast) -> bool {
    let app = toast.app_name.to_lowercase();
    settings.blocked_apps.iter().any(|blocked| blocked.trim().to_lowercase() == app)
}

/// Lance la surveillance des notifications du système.
pub fn start(app: AppHandle) {
    #[cfg(windows)]
    windows_watch::start(app);
    #[cfg(target_os = "linux")]
    linux::start(app);
    #[cfg(not(any(windows, target_os = "linux")))]
    drop(app);
}

/// `allowed`, `denied` ou `unspecified`.
#[tauri::command]
pub fn get_notification_access() -> Result<Access, String> {
    #[cfg(windows)]
    {
        listener::access_status()
    }
    #[cfg(not(windows))]
    {
        Ok(Access::Allowed)
    }
}

/// Demande l'autorisation au système. Windows : sur un thread dédié avec COM initialisé.
/// Linux : aucune autorisation à demander.
#[tauri::command]
pub async fn request_notification_access() -> Result<Access, String> {
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(|| {
            let _com = crate::com::ComGuard::new().map_err(|e| e.to_string())?;
            listener::request_access()
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(not(windows))]
    {
        Ok(Access::Allowed)
    }
}

/// Windows : le centre de notifications est relu toutes les secondes.
#[cfg(windows)]
mod windows_watch {
    use std::collections::HashSet;
    use std::thread;
    use std::time::Duration;

    use tauri::{AppHandle, Emitter, Manager};

    use super::{is_blocked, listener, Access, Toast, EVENT_NOTIFICATION};
    use crate::com::ComGuard;
    use crate::settings::SettingsState;

    const POLL_INTERVAL: Duration = Duration::from_millis(1000);
    /// Quand la fonction est désactivée, on ne vérifie les réglages que de temps en temps.
    const DISABLED_INTERVAL: Duration = Duration::from_secs(3);

    /// Garde les toasts jamais vus, dans l'ordre d'arrivée, et mémorise tous les identifiants courants.
    pub(super) fn take_new(seen: &mut HashSet<u32>, current: Vec<Toast>) -> Vec<Toast> {
        let current_ids: HashSet<u32> = current.iter().map(|toast| toast.id).collect();
        let mut fresh: Vec<Toast> = current.into_iter().filter(|toast| !seen.contains(&toast.id)).collect();
        fresh.sort_by_key(|toast| toast.created_ms);
        // On ne garde que les identifiants encore présents : l'ensemble ne grossit pas indéfiniment.
        *seen = current_ids;
        fresh
    }

    pub fn start(app: AppHandle) {
        thread::spawn(move || {
            let Ok(_com) = ComGuard::new() else {
                eprintln!("notification watcher: COM unavailable");
                return;
            };

            let mut seen: HashSet<u32> = HashSet::new();
            // Au premier passage, les notifications déjà présentes sont mémorisées sans être affichées.
            let mut primed = false;

            loop {
                let settings = app.state::<SettingsState>().snapshot().notifications;

                if !settings.enabled {
                    primed = false;
                    seen.clear();
                    thread::sleep(DISABLED_INTERVAL);
                    continue;
                }

                if listener::access_status() == Ok(Access::Allowed) {
                    match listener::fetch_toasts() {
                        Ok(current) if !primed => {
                            seen = current.iter().map(|toast| toast.id).collect();
                            primed = true;
                        }
                        Ok(current) => {
                            for toast in take_new(&mut seen, current) {
                                if is_blocked(&settings, &toast) {
                                    continue;
                                }
                                if settings.sound_enabled {
                                    crate::sound::play_notification();
                                }
                                let _ = app.emit(EVENT_NOTIFICATION, toast);
                            }
                        }
                        Err(error) => eprintln!("notification fetch failed: {error}"),
                    }
                } else {
                    // Pas d'autorisation : on repart de zéro quand elle sera accordée.
                    primed = false;
                }

                thread::sleep(POLL_INTERVAL);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toast(id: u32, app: &str, created_ms: i64) -> Toast {
        Toast { id, app_name: app.into(), title: "t".into(), body: String::new(), created_ms }
    }

    #[cfg(windows)]
    #[test]
    fn only_unseen_toasts_are_returned_in_arrival_order() {
        use std::collections::HashSet;
        let mut seen: HashSet<u32> = [1, 2].into_iter().collect();
        let fresh = windows_watch::take_new(&mut seen, vec![toast(3, "A", 30), toast(1, "A", 10), toast(4, "B", 20)]);

        assert_eq!(fresh.iter().map(|t| t.id).collect::<Vec<_>>(), vec![4, 3]);
        assert_eq!(seen, [1, 3, 4].into_iter().collect());
    }

    #[cfg(windows)]
    #[test]
    fn dismissed_toasts_are_forgotten() {
        use std::collections::HashSet;
        let mut seen: HashSet<u32> = [1, 2, 3].into_iter().collect();
        windows_watch::take_new(&mut seen, vec![toast(3, "A", 1)]);
        assert_eq!(seen, [3].into_iter().collect());
    }

    #[test]
    fn blocked_apps_match_case_insensitively() {
        let settings = NotificationSettings { blocked_apps: vec![" Discord ".into()], ..Default::default() };
        assert!(is_blocked(&settings, &toast(1, "discord", 0)));
        assert!(!is_blocked(&settings, &toast(2, "Teams", 0)));
    }
}
