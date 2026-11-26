//! Média en cours sous Linux, via MPRIS (D-Bus). Spotify, VLC, Firefox, Chromium, mpv, Rhythmbox
//! et la plupart des lecteurs exposent ce standard. Le thread relit l'état une fois par seconde.

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use tauri::{AppHandle, Emitter, Manager};
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{OwnedValue, Value};

use super::controls::MediaPosition;
use super::cover::{file_path_from_url, sniff_mime};
use super::{MediaInfo, MediaState, EVENT_MEDIA_CHANGED};

const POLL_INTERVAL: Duration = Duration::from_millis(1000);
const PREFIX: &str = "org.mpris.MediaPlayer2.";
const OBJECT: &str = "/org/mpris/MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";
const MAX_COVER_BYTES: usize = 3 * 1024 * 1024;

/// Ce qu'un lecteur annonce à un instant donné.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub bus_name: String,
    pub status: String,
    pub title: String,
    pub artist: String,
    pub art_url: Option<String>,
    pub track_id: Option<String>,
    pub length_micros: Option<i64>,
}

impl Snapshot {
    pub fn playing(&self) -> bool {
        self.status == "Playing"
    }

    fn has_track(&self) -> bool {
        !self.title.is_empty() || !self.artist.is_empty()
    }
}

/// Lecteur à afficher : celui qui joue, sinon celui en pause, sinon le premier avec un morceau.
pub fn choose(snapshots: Vec<Snapshot>) -> Option<Snapshot> {
    let mut candidates: Vec<Snapshot> = snapshots.into_iter().filter(Snapshot::has_track).collect();
    candidates.sort_by_key(|snapshot| match snapshot.status.as_str() {
        "Playing" => 0,
        "Paused" => 1,
        _ => 2,
    });
    candidates.into_iter().next()
}

fn as_string(value: &Value) -> Option<String> {
    match value {
        Value::Str(text) => Some(text.as_str().to_owned()),
        Value::Value(inner) => as_string(inner),
        _ => None,
    }
}

fn as_i64(value: &Value) -> Option<i64> {
    match value {
        Value::I64(number) => Some(*number),
        Value::U64(number) => i64::try_from(*number).ok(),
        Value::I32(number) => Some(i64::from(*number)),
        Value::U32(number) => Some(i64::from(*number)),
        Value::Value(inner) => as_i64(inner),
        _ => None,
    }
}

/// `xesam:artist` est une liste de noms ; certains lecteurs envoient une simple chaîne.
fn as_names(value: &Value) -> String {
    match value {
        Value::Array(names) => names.iter().filter_map(as_string).collect::<Vec<_>>().join(", "),
        Value::Value(inner) => as_names(inner),
        other => as_string(other).unwrap_or_default(),
    }
}

fn as_path(value: &Value) -> Option<String> {
    match value {
        Value::ObjectPath(path) => Some(path.as_str().to_owned()),
        Value::Str(text) => Some(text.as_str().to_owned()),
        Value::Value(inner) => as_path(inner),
        _ => None,
    }
}

fn snapshot_from(bus_name: &str, status: String, metadata: &HashMap<String, OwnedValue>) -> Snapshot {
    let get = |key: &str| metadata.get(key).map(|value| &**value);
    Snapshot {
        bus_name: bus_name.to_string(),
        status,
        title: get("xesam:title").and_then(as_string).unwrap_or_default(),
        artist: get("xesam:artist").map(as_names).unwrap_or_default(),
        art_url: get("mpris:artUrl").and_then(as_string).filter(|url| !url.is_empty()),
        track_id: get("mpris:trackid").and_then(as_path),
        length_micros: get("mpris:length").and_then(as_i64).filter(|length| *length > 0),
    }
}

fn proxy(connection: &Connection, bus_name: &str) -> Result<Proxy<'static>, String> {
    Proxy::new(connection, bus_name.to_string(), OBJECT, PLAYER).map_err(|e| e.to_string())
}

fn player_names(connection: &Connection) -> Result<Vec<String>, String> {
    let bus = Proxy::new(connection, "org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus")
        .map_err(|e| e.to_string())?;
    let names: Vec<String> = bus.call("ListNames", &()).map_err(|e| e.to_string())?;
    Ok(names.into_iter().filter(|name| name.starts_with(PREFIX)).collect())
}

fn read(connection: &Connection, bus_name: &str) -> Option<Snapshot> {
    let player = proxy(connection, bus_name).ok()?;
    let status: String = player.get_property("PlaybackStatus").ok()?;
    let metadata: HashMap<String, OwnedValue> = player.get_property("Metadata").ok()?;
    Some(snapshot_from(bus_name, status, &metadata))
}

/// Lecteur actuellement choisi.
pub fn current(connection: &Connection) -> Option<Snapshot> {
    let names = player_names(connection).ok()?;
    choose(names.iter().filter_map(|name| read(connection, name)).collect())
}

fn connect() -> Result<Connection, String> {
    Connection::session().map_err(|e| format!("Bus de session D-Bus indisponible : {e}"))
}

const NO_MEDIA: &str = "Aucun média en cours";

fn with_player<T>(action: impl FnOnce(&Connection, &Snapshot) -> Result<T, String>) -> Result<T, String> {
    let connection = connect()?;
    let snapshot = current(&connection).ok_or_else(|| NO_MEDIA.to_string())?;
    action(&connection, &snapshot)
}

fn call(connection: &Connection, bus_name: &str, method: &str) -> Result<(), String> {
    proxy(connection, bus_name)?.call_method(method, &()).map(drop).map_err(|e| e.to_string())
}

pub fn toggle_play_pause() -> Result<(), String> {
    with_player(|connection, player| call(connection, &player.bus_name, "PlayPause"))
}

pub fn next() -> Result<(), String> {
    with_player(|connection, player| call(connection, &player.bus_name, "Next"))
}

pub fn previous() -> Result<(), String> {
    with_player(|connection, player| call(connection, &player.bus_name, "Previous"))
}

fn position_micros(connection: &Connection, bus_name: &str) -> Option<i64> {
    proxy(connection, bus_name).ok()?.get_property::<i64>("Position").ok()
}

pub fn seek(seconds: f64) -> Result<(), String> {
    with_player(|connection, player| {
        let target = (seconds.max(0.0) * 1_000_000.0) as i64;
        let player_proxy = proxy(connection, &player.bus_name)?;
        if let Some(track_id) = &player.track_id {
            if let Ok(path) = zbus::zvariant::ObjectPath::try_from(track_id.as_str()) {
                if player_proxy.call_method("SetPosition", &(path, target)).is_ok() {
                    return Ok(());
                }
            }
        }
        // Sans identifiant de piste, on se déplace d'un écart relatif à la position actuelle.
        let current = position_micros(connection, &player.bus_name).unwrap_or(0);
        player_proxy.call_method("Seek", &(target - current)).map(drop).map_err(|e| e.to_string())
    })
}

pub fn position() -> Result<Option<MediaPosition>, String> {
    match with_player(|connection, player| {
        Ok(MediaPosition {
            position_seconds: position_micros(connection, &player.bus_name).unwrap_or(0).max(0) as f64 / 1_000_000.0,
            duration_seconds: player.length_micros.map(|length| length as f64 / 1_000_000.0),
            playing: player.playing(),
        })
    }) {
        Ok(position) => Ok(Some(position)),
        Err(message) if message == NO_MEDIA => Ok(None),
        Err(message) => Err(message),
    }
}

/// Image de la pochette : fichier local ou adresse web.
fn fetch_cover(art_url: &str) -> Option<(String, Vec<u8>)> {
    let bytes = if let Some(path) = file_path_from_url(art_url) {
        std::fs::read(path).ok()?
    } else if art_url.starts_with("http://") || art_url.starts_with("https://") {
        let url = art_url.to_string();
        tauri::async_runtime::block_on(async move {
            let response = reqwest::get(url).await.ok()?.error_for_status().ok()?;
            response.bytes().await.ok().map(|bytes| bytes.to_vec())
        })?
    } else {
        return None;
    };
    if bytes.is_empty() || bytes.len() > MAX_COVER_BYTES {
        return None;
    }
    let mime = sniff_mime(&bytes)?;
    Some((mime.to_string(), bytes))
}

struct CoverCache {
    art_url: String,
    cover: Option<(String, Vec<u8>)>,
}

fn media_info(snapshot: &Snapshot, cache: &mut Option<CoverCache>) -> MediaInfo {
    if let Some(url) = &snapshot.art_url {
        if cache.as_ref().map(|c| c.art_url.as_str()) != Some(url.as_str()) {
            *cache = Some(CoverCache { art_url: url.clone(), cover: fetch_cover(url) });
        }
    } else {
        *cache = None;
    }
    let cover = cache.as_ref().and_then(|c| c.cover.as_ref());
    MediaInfo {
        title: snapshot.title.clone(),
        artist: snapshot.artist.clone(),
        playing: snapshot.playing(),
        cover_base64: cover.map(|(_, bytes)| STANDARD.encode(bytes)),
        cover_mime: cover.map(|(mime, _)| mime.clone()),
    }
}

/// Lance la surveillance du média en cours dans un thread dédié.
pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut connection: Option<Connection> = None;
        let mut cache: Option<CoverCache> = None;
        let mut last: Option<MediaInfo> = None;

        loop {
            if connection.is_none() {
                connection = Connection::session().ok();
            }
            let current_info = connection
                .as_ref()
                .and_then(current)
                .map(|snapshot| media_info(&snapshot, &mut cache));

            if current_info != last {
                app.state::<MediaState>().set(current_info.clone());
                let _ = app.emit(EVENT_MEDIA_CHANGED, current_info.clone());
                last = current_info;
            }
            thread::sleep(POLL_INTERVAL);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(name: &str, status: &str, title: &str) -> Snapshot {
        Snapshot { bus_name: name.into(), status: status.into(), title: title.into(), ..Default::default() }
    }

    #[test]
    fn the_playing_player_wins_over_a_paused_one() {
        let chosen = choose(vec![snapshot("a", "Paused", "Un"), snapshot("b", "Playing", "Deux")]).unwrap();
        assert_eq!(chosen.bus_name, "b");
    }

    #[test]
    fn players_without_a_track_are_ignored() {
        assert!(choose(vec![snapshot("a", "Stopped", "")]).is_none());
        let chosen = choose(vec![snapshot("a", "Stopped", ""), snapshot("b", "Paused", "Titre")]).unwrap();
        assert_eq!(chosen.bus_name, "b");
    }

    #[test]
    fn metadata_is_converted() {
        let mut metadata: HashMap<String, OwnedValue> = HashMap::new();
        metadata.insert("xesam:title".into(), Value::new("Chanson").try_into().unwrap());
        metadata.insert("xesam:artist".into(), Value::new(vec!["A", "B"]).try_into().unwrap());
        metadata.insert("mpris:length".into(), Value::new(215_000_000u64).try_into().unwrap());
        let snapshot = snapshot_from("org.mpris.MediaPlayer2.test", "Playing".into(), &metadata);
        assert_eq!(snapshot.title, "Chanson");
        assert_eq!(snapshot.artist, "A, B");
        assert_eq!(snapshot.length_micros, Some(215_000_000));
        assert!(snapshot.playing());
    }
}
