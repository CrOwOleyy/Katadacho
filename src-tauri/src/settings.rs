//! Réglages de l'application, stockés en JSON dans la table `settings`.
//! Chaque groupe a des valeurs par défaut : ajouter un champ plus tard n'invalide pas
//! les réglages déjà enregistrés.

use std::sync::RwLock;

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::db::Database;

pub const EVENT_SETTINGS_CHANGED: &str = "settings-changed";
const STORAGE_KEY: &str = "app";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct AnimationSettings {
    /// Désactive toutes les animations de mouvement si `false`.
    pub enabled: bool,
    /// Multiplicateur de vitesse : 1.0 normal, 2.0 deux fois plus rapide, 0.5 deux fois plus lent.
    pub speed: f32,
    /// Survol nécessaire avant l'ouverture du menu, en millisecondes.
    pub open_delay_ms: u32,
    /// Délai avant fermeture quand la souris quitte le menu, en millisecondes.
    pub close_delay_ms: u32,
    /// Durée du fondu entre deux onglets, en millisecondes.
    pub tab_transition_ms: u32,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            speed: 1.0,
            open_delay_ms: 300,
            close_delay_ms: 450,
            tab_transition_ms: 140,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct IslandSettings {
    /// Secondes avant que l'Island se cache une fois repliée. 0 = ne jamais se cacher.
    pub auto_hide_seconds: u32,
    pub show_media_in_pill: bool,
}

impl Default for IslandSettings {
    fn default() -> Self {
        Self {
            auto_hide_seconds: 15,
            show_media_in_pill: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct TimerSettings {
    pub focus_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub sessions_before_long_break: u32,
    /// Enchaîne automatiquement la phase suivante sans attendre un clic.
    pub auto_start_next: bool,
    pub sound_enabled: bool,
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self {
            focus_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            sessions_before_long_break: 4,
            auto_start_next: false,
            sound_enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationSettings {
    pub enabled: bool,
    /// Durée d'affichage d'une notification dans l'Island, en secondes.
    pub display_seconds: u32,
    /// Affiche le texte du message, pas seulement le nom de l'application.
    pub show_body: bool,
    pub sound_enabled: bool,
    /// Noms d'applications dont les notifications sont ignorées.
    pub blocked_apps: Vec<String>,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            display_seconds: 6,
            show_body: true,
            sound_enabled: false,
            blocked_apps: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct ClipboardSettings {
    /// Repère les clés d'API, jetons et mots de passe copiés à la main.
    pub detect_secrets: bool,
    /// Minutes avant l'effacement automatique d'un secret non épinglé.
    pub secret_ttl_minutes: u32,
}

impl Default for ClipboardSettings {
    fn default() -> Self {
        Self {
            detect_secrets: true,
            secret_ttl_minutes: 2,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub animations: AnimationSettings,
    pub island: IslandSettings,
    pub timer: TimerSettings,
    pub notifications: NotificationSettings,
    pub clipboard: ClipboardSettings,
}

impl Settings {
    /// Ramène chaque valeur dans une plage sûre, pour qu'un réglage absurde ne casse pas l'interface.
    pub fn sanitized(mut self) -> Self {
        let a = &mut self.animations;
        a.speed = a.speed.clamp(0.25, 3.0);
        a.open_delay_ms = a.open_delay_ms.min(3000);
        a.close_delay_ms = a.close_delay_ms.min(5000);
        a.tab_transition_ms = a.tab_transition_ms.min(1000);

        self.island.auto_hide_seconds = self.island.auto_hide_seconds.min(3600);

        let t = &mut self.timer;
        t.focus_minutes = t.focus_minutes.clamp(1, 180);
        t.short_break_minutes = t.short_break_minutes.clamp(1, 60);
        t.long_break_minutes = t.long_break_minutes.clamp(1, 120);
        t.sessions_before_long_break = t.sessions_before_long_break.clamp(1, 12);

        let n = &mut self.notifications;
        n.display_seconds = n.display_seconds.clamp(2, 60);
        n.blocked_apps.retain(|name| !name.trim().is_empty());

        self.clipboard.secret_ttl_minutes = self.clipboard.secret_ttl_minutes.clamp(1, 1440);
        self
    }
}

/// Copie en mémoire des réglages, lisible par les modules Rust (minuteur, notifications).
pub struct SettingsState(pub RwLock<Settings>);

impl SettingsState {
    pub fn snapshot(&self) -> Settings {
        self.0.read().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

pub fn load(database: &Database) -> Settings {
    let stored: Option<String> = database
        .with(|conn| {
            conn.query_row("SELECT value FROM settings WHERE key = ?1", params![STORAGE_KEY], |row| row.get(0))
                .optional()
        })
        .unwrap_or(None);

    stored
        .and_then(|json| serde_json::from_str::<Settings>(&json).ok())
        .unwrap_or_default()
        .sanitized()
}

#[tauri::command]
pub fn get_settings(state: State<'_, SettingsState>) -> Settings {
    state.snapshot()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    database: State<'_, Database>,
    state: State<'_, SettingsState>,
    settings: Settings,
) -> Result<Settings, String> {
    let settings = settings.sanitized();
    let json = serde_json::to_string(&settings).map_err(|e| e.to_string())?;

    database.with(|conn| {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![STORAGE_KEY, json],
        )
    })?;

    *state.0.write().unwrap_or_else(|e| e.into_inner()) = settings.clone();
    let _ = app.emit(EVENT_SETTINGS_CHANGED, settings.clone());
    Ok(settings)
}


/// Remet tous les réglages à leurs valeurs par défaut et notifie le frontend.
#[tauri::command]
pub fn reset_settings(
    app: AppHandle,
    database: State<'_, Database>,
    state: State<'_, SettingsState>,
) -> Result<Settings, String> {
    save_settings(app, database, state, Settings::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let settings: Settings = serde_json::from_str(r#"{"timer":{"focusMinutes":50}}"#).unwrap();
        assert_eq!(settings.timer.focus_minutes, 50);
        assert_eq!(settings.timer.short_break_minutes, 5);
        assert_eq!(settings.animations.open_delay_ms, 300);
    }

    #[test]
    fn sanitizing_clamps_out_of_range_values() {
        let mut settings = Settings::default();
        settings.animations.speed = 99.0;
        settings.timer.focus_minutes = 0;
        settings.notifications.display_seconds = 0;
        settings.notifications.blocked_apps = vec!["  ".into(), "Discord".into()];

        let clean = settings.sanitized();
        assert_eq!(clean.animations.speed, 3.0);
        assert_eq!(clean.timer.focus_minutes, 1);
        assert_eq!(clean.notifications.display_seconds, 2);
        assert_eq!(clean.notifications.blocked_apps, vec!["Discord".to_string()]);
    }
}
