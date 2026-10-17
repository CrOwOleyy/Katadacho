//! Commandes de lecture envoyées à l'application multimédia en cours : lecture/pause, piste
//! suivante ou précédente, déplacement dans le morceau.

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaPosition {
    pub position_seconds: f64,
    /// `None` pour un flux sans durée connue, par exemple une radio en direct.
    pub duration_seconds: Option<f64>,
    pub playing: bool,
}

#[tauri::command]
pub async fn media_toggle_play_pause() -> Result<(), String> {
    imp::toggle_play_pause().await
}

#[tauri::command]
pub async fn media_next() -> Result<(), String> {
    imp::next().await
}

#[tauri::command]
pub async fn media_previous() -> Result<(), String> {
    imp::previous().await
}

/// Déplace la lecture à `seconds` depuis le début du morceau.
#[tauri::command]
pub async fn media_seek(seconds: f64) -> Result<(), String> {
    imp::seek(seconds).await
}

/// Position actuelle et durée. Renvoie `None` quand aucun média n'est en cours.
#[tauri::command]
pub async fn media_position() -> Result<Option<MediaPosition>, String> {
    imp::position().await
}

#[cfg(windows)]
mod imp {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };

    use super::MediaPosition;
    use crate::com::ComGuard;

    /// Un TimeSpan Windows compte des intervalles de 100 nanosecondes.
    const TICKS_PER_SECOND: f64 = 10_000_000.0;

    /// Exécute `action` sur la session en cours, sur un thread bloquant avec COM initialisé.
    async fn with_session<T, F>(action: F) -> Result<T, String>
    where
        T: Send + 'static,
        F: FnOnce(&GlobalSystemMediaTransportControlsSession) -> Result<T, String> + Send + 'static,
    {
        tauri::async_runtime::spawn_blocking(move || {
            let _com = ComGuard::new().map_err(|e| e.to_string())?;
            let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .and_then(|operation| operation.join())
                .map_err(|e| e.to_string())?;
            let session = manager
                .GetCurrentSession()
                .map_err(|_| "Aucun média en cours".to_string())?;
            action(&session)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Convertit le résultat `bool` d'une commande en erreur lisible quand l'application la refuse.
    fn accepted(result: bool, label: &str) -> Result<(), String> {
        if result {
            Ok(())
        } else {
            Err(format!("L'application ne permet pas : {label}"))
        }
    }

    pub async fn toggle_play_pause() -> Result<(), String> {
        with_session(|session| {
            let result = session
                .TryTogglePlayPauseAsync()
                .and_then(|operation| operation.join())
                .map_err(|e| e.to_string())?;
            accepted(result, "lecture ou pause")
        })
        .await
    }

    pub async fn next() -> Result<(), String> {
        with_session(|session| {
            let result = session
                .TrySkipNextAsync()
                .and_then(|operation| operation.join())
                .map_err(|e| e.to_string())?;
            accepted(result, "piste suivante")
        })
        .await
    }

    pub async fn previous() -> Result<(), String> {
        with_session(|session| {
            let result = session
                .TrySkipPreviousAsync()
                .and_then(|operation| operation.join())
                .map_err(|e| e.to_string())?;
            accepted(result, "piste précédente")
        })
        .await
    }

    pub async fn seek(seconds: f64) -> Result<(), String> {
        with_session(move |session| {
            let ticks = (seconds.max(0.0) * TICKS_PER_SECOND) as i64;
            let result = session
                .TryChangePlaybackPositionAsync(ticks)
                .and_then(|operation| operation.join())
                .map_err(|e| e.to_string())?;
            accepted(result, "déplacement dans le morceau")
        })
        .await
    }

    pub async fn position() -> Result<Option<MediaPosition>, String> {
        let result = with_session(|session| {
            let timeline = session.GetTimelineProperties().map_err(|e| e.to_string())?;
            let position = timeline.Position().map_err(|e| e.to_string())?.Duration;
            let start = timeline.StartTime().map(|t| t.Duration).unwrap_or(0);
            let end = timeline.EndTime().map(|t| t.Duration).unwrap_or(0);

            let playing = session
                .GetPlaybackInfo()
                .and_then(|info| info.PlaybackStatus())
                .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
                .unwrap_or(false);

            let duration = end - start;
            Ok(MediaPosition {
                position_seconds: ((position - start).max(0)) as f64 / TICKS_PER_SECOND,
                duration_seconds: (duration > 0).then(|| duration as f64 / TICKS_PER_SECOND),
                playing,
            })
        })
        .await;

        match result {
            Ok(position) => Ok(Some(position)),
            // Pas de session : état normal, pas une erreur pour l'interface.
            Err(message) if message == "Aucun média en cours" => Ok(None),
            Err(message) => Err(message),
        }
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod imp {
    use super::MediaPosition;
    use crate::platform::unsupported;

    pub async fn toggle_play_pause() -> Result<(), String> {
        Err(unsupported("Le contrôle du média"))
    }

    pub async fn next() -> Result<(), String> {
        Err(unsupported("Le contrôle du média"))
    }

    pub async fn previous() -> Result<(), String> {
        Err(unsupported("Le contrôle du média"))
    }

    pub async fn seek(_seconds: f64) -> Result<(), String> {
        Err(unsupported("Le contrôle du média"))
    }

    pub async fn position() -> Result<Option<MediaPosition>, String> {
        Ok(None)
    }
}

/// Linux : MPRIS sur le bus de session D-Bus, lu sur un thread bloquant.
#[cfg(target_os = "linux")]
mod imp {
    use super::MediaPosition;
    use crate::media::mpris;

    async fn blocking<T: Send + 'static>(task: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
        tauri::async_runtime::spawn_blocking(task).await.map_err(|e| e.to_string())?
    }

    pub async fn toggle_play_pause() -> Result<(), String> {
        blocking(mpris::toggle_play_pause).await
    }

    pub async fn next() -> Result<(), String> {
        blocking(mpris::next).await
    }

    pub async fn previous() -> Result<(), String> {
        blocking(mpris::previous).await
    }

    pub async fn seek(seconds: f64) -> Result<(), String> {
        blocking(move || mpris::seek(seconds)).await
    }

    pub async fn position() -> Result<Option<MediaPosition>, String> {
        blocking(mpris::position).await
    }
}
