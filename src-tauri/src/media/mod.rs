//! Média en cours de lecture, via les contrôles multimédias système de Windows
//! (GlobalSystemMediaTransportControls). Fonctionne avec Spotify, les navigateurs, VLC, etc.

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionMediaProperties,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;

use crate::controls::audio::ComGuard;

pub mod controls;
pub mod levels;

pub const EVENT_MEDIA_CHANGED: &str = "media-changed";
const POLL_INTERVAL: Duration = Duration::from_millis(1000);
/// Certaines applications fournissent la pochette quelques secondes après le titre.
const COVER_RETRIES: u32 = 8;
const MAX_COVER_BYTES: u64 = 3 * 1024 * 1024;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub cover_base64: Option<String>,
    pub cover_mime: Option<String>,
}

/// Dernier état connu, pour que le frontend puisse le relire au montage sans attendre un événement.
#[derive(Default)]
pub struct MediaState(Mutex<Option<MediaInfo>>);

#[tauri::command]
pub fn get_media_state(state: State<'_, MediaState>) -> Option<MediaInfo> {
    state.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

struct CoverCache {
    track_key: String,
    cover: Option<(String, Vec<u8>)>,
    attempts: u32,
}

fn read_cover(properties: &GlobalSystemMediaTransportControlsSessionMediaProperties) -> Option<(String, Vec<u8>)> {
    let reference = properties.Thumbnail().ok()?;
    let stream = reference.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()?;
    if size == 0 || size > MAX_COVER_BYTES {
        return None;
    }
    let size = size as u32;

    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0).ok()?).ok()?;
    reader.LoadAsync(size).ok()?.join().ok()?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes).ok()?;

    let mime = stream.ContentType().ok()?.to_string();
    Some((if mime.is_empty() { "image/jpeg".to_string() } else { mime }, bytes))
}

fn snapshot(session: &GlobalSystemMediaTransportControlsSession, cache: &mut Option<CoverCache>) -> Option<MediaInfo> {
    let properties = session.TryGetMediaPropertiesAsync().ok()?.join().ok()?;
    let title = properties.Title().ok()?.to_string();
    let artist = properties.Artist().map(|a| a.to_string()).unwrap_or_default();
    if title.is_empty() && artist.is_empty() {
        return None;
    }

    let playing = session
        .GetPlaybackInfo()
        .and_then(|info| info.PlaybackStatus())
        .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
        .unwrap_or(false);

    let track_key = format!("{title}\u{1f}{artist}");
    let needs_read = match cache {
        Some(existing) if existing.track_key == track_key => {
            existing.cover.is_none() && existing.attempts < COVER_RETRIES
        }
        _ => true,
    };
    if needs_read {
        let attempts = match cache {
            Some(existing) if existing.track_key == track_key => existing.attempts + 1,
            _ => 1,
        };
        *cache = Some(CoverCache { track_key, cover: read_cover(&properties), attempts });
    }

    let cover = cache.as_ref().and_then(|c| c.cover.as_ref());
    Some(MediaInfo {
        title,
        artist,
        playing,
        cover_base64: cover.map(|(_, bytes)| STANDARD.encode(bytes)),
        cover_mime: cover.map(|(mime, _)| mime.clone()),
    })
}

/// Lance la surveillance du média en cours dans un thread dédié.
pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let Ok(_com) = ComGuard::new() else {
            eprintln!("media watcher: COM unavailable");
            return;
        };
        let Some(manager) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .ok()
            .and_then(|operation| operation.join().ok())
        else {
            eprintln!("media watcher: session manager unavailable");
            return;
        };

        let mut cache: Option<CoverCache> = None;
        let mut last: Option<MediaInfo> = None;

        loop {
            let current = manager
                .GetCurrentSession()
                .ok()
                .and_then(|session| snapshot(&session, &mut cache));

            if current != last {
                *app.state::<MediaState>().0.lock().unwrap_or_else(|e| e.into_inner()) = current.clone();
                let _ = app.emit(EVENT_MEDIA_CHANGED, current.clone());
                last = current;
            }
            thread::sleep(POLL_INTERVAL);
        }
    });
}
