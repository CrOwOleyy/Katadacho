//! Contrôles système sous Linux. Chaque contrôle passe par l'outil standard de la distribution :
//! `pactl` (PulseAudio et PipeWire), `/sys/class/backlight` avec `busctl` (logind) ou `brightnessctl`,
//! `nmcli` (NetworkManager) et `bluetoothctl` (BlueZ). Un outil absent donne une erreur lisible.

use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

use super::mixer::{self, Stream};
use super::parse;
use super::types::{AppVolume, AudioState, NetworkBluetoothState};

const BACKLIGHT_DIR: &str = "/sys/class/backlight";

/// Lance `program` et retourne sa sortie standard. Une erreur nomme l'outil manquant.
fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program).args(args).output().map_err(|error| {
        if error.kind() == ErrorKind::NotFound {
            format!("`{program}` est introuvable : installe-le pour utiliser cette fonction")
        } else {
            format!("`{program}` : {error}")
        }
    })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("`{program}` a échoué : {}", stderr.trim()))
    }
}

pub fn get_audio_state() -> Result<AudioState, String> {
    let sinks = run("pactl", &["--format=json", "list", "sinks"])?;
    let default_sink = run("pactl", &["get-default-sink"])?;
    parse::audio_state(&sinks, default_sink.trim())
}

pub fn set_master_volume(level: u8) -> Result<(), String> {
    run("pactl", &["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", level.min(100))]).map(drop)
}

pub fn toggle_mute() -> Result<bool, String> {
    run("pactl", &["set-sink-mute", "@DEFAULT_SINK@", "toggle"])?;
    Ok(parse::is_muted(&run("pactl", &["get-sink-mute", "@DEFAULT_SINK@"])?))
}

pub fn set_default_audio_device(device_id: &str) -> Result<(), String> {
    run("pactl", &["set-default-sink", device_id])?;
    // Les lectures déjà en cours restent sur l'ancienne sortie : on les déplace vers la nouvelle.
    if let Ok(inputs) = run("pactl", &["list", "short", "sink-inputs"]) {
        for stream in inputs.lines().filter_map(|line| line.split_whitespace().next()) {
            let _ = run("pactl", &["move-sink-input", stream, device_id]);
        }
    }
    Ok(())
}

/// Premier écran rétroéclairé : nom du pilote, valeur actuelle et maximum.
fn backlight() -> Result<(String, u32, u32), String> {
    let mut entries: Vec<PathBuf> = fs::read_dir(BACKLIGHT_DIR)
        .map_err(|_| "Aucun écran à luminosité réglable sur cet appareil".to_string())?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();

    let read = |path: &PathBuf, file: &str| -> Option<u32> {
        fs::read_to_string(path.join(file)).ok()?.trim().parse().ok()
    };
    entries
        .iter()
        .find_map(|path| {
            let name = path.file_name()?.to_str()?.to_string();
            Some((name, read(path, "brightness")?, read(path, "max_brightness")?))
        })
        .ok_or_else(|| "Aucun écran à luminosité réglable sur cet appareil".to_string())
}

pub fn get_brightness() -> Result<u8, String> {
    let (_, raw, max) = backlight()?;
    Ok(parse::brightness_percent(raw, max))
}

pub fn set_brightness(level: u8) -> Result<(), String> {
    let (name, _, max) = backlight()?;
    let raw = parse::brightness_raw(level, max);

    // 1. Écriture directe : possible quand une règle udev donne l'accès au groupe de l'utilisateur.
    let path = PathBuf::from(BACKLIGHT_DIR).join(&name).join("brightness");
    if fs::write(&path, raw.to_string()).is_ok() {
        return Ok(());
    }
    // 2. logind autorise la session active à régler la luminosité sans mot de passe.
    let value = raw.to_string();
    if run(
        "busctl",
        &[
            "call",
            "org.freedesktop.login1",
            "/org/freedesktop/login1/session/auto",
            "org.freedesktop.login1.Session",
            "SetBrightness",
            "ssu",
            "backlight",
            &name,
            &value,
        ],
    )
    .is_ok()
    {
        return Ok(());
    }
    // 3. Dernier recours : brightnessctl, s'il est installé.
    run("brightnessctl", &["--device", &name, "set", &format!("{}%", level.max(1))])
        .map(drop)
        .map_err(|_| "Impossible de régler la luminosité : droits insuffisants (logind indisponible)".to_string())
}

pub fn get_network_bt_state() -> Result<NetworkBluetoothState, String> {
    let wifi_enabled = run("nmcli", &["radio", "wifi"]).ok().and_then(|out| parse::radio_enabled(&out));
    let wifi = run("nmcli", &["--terse", "--fields", "ACTIVE,SSID,SIGNAL", "device", "wifi"])
        .ok()
        .and_then(|out| parse::active_wifi(&out));
    let bluetooth_enabled = run("bluetoothctl", &["show"]).ok().and_then(|out| parse::bluetooth_powered(&out));
    let bluetooth_devices = if bluetooth_enabled == Some(true) {
        run("bluetoothctl", &["devices", "Connected"])
            .map(|out| parse::bluetooth_devices(&out))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(NetworkBluetoothState {
        wifi_enabled,
        wifi_ssid: wifi.as_ref().map(|(ssid, _)| ssid.clone()),
        wifi_signal: wifi.map(|(_, signal)| signal),
        bluetooth_enabled,
        bluetooth_devices,
    })
}

fn state_word(enable: bool) -> &'static str {
    if enable {
        "on"
    } else {
        "off"
    }
}

pub fn toggle_wifi(enable: bool) -> Result<(), String> {
    run("nmcli", &["radio", "wifi", state_word(enable)])
        .or_else(|_| run("rfkill", &[if enable { "unblock" } else { "block" }, "wifi"]))
        .map(drop)
}

pub fn toggle_bluetooth(enable: bool) -> Result<(), String> {
    run("bluetoothctl", &["power", state_word(enable)])
        .or_else(|_| run("rfkill", &[if enable { "unblock" } else { "block" }, "bluetooth"]))
        .map(drop)
}

/// Flux audio des applications, avec leur numéro pactl.
fn sink_inputs() -> Result<Vec<(u32, Stream)>, String> {
    parse::sink_inputs(&run("pactl", &["--format=json", "list", "sink-inputs"])?)
}

pub fn get_app_volumes() -> Result<Vec<AppVolume>, String> {
    Ok(mixer::group(sink_inputs()?.into_iter().map(|(_, stream)| stream).collect()))
}

fn matching(id: &str) -> Result<Vec<(u32, Stream)>, String> {
    let found: Vec<(u32, Stream)> = sink_inputs()?.into_iter().filter(|(_, stream)| stream.key == id).collect();
    if found.is_empty() {
        Err("Cette application ne joue plus de son".to_string())
    } else {
        Ok(found)
    }
}

pub fn set_app_volume(id: &str, level: u8) -> Result<(), String> {
    let percent = format!("{}%", level.min(100));
    for (index, _) in matching(id)? {
        run("pactl", &["set-sink-input-volume", &index.to_string(), &percent])?;
    }
    Ok(())
}

/// Inverse le mute de tous les flux de l'application et retourne le nouvel état.
pub fn toggle_app_mute(id: &str) -> Result<bool, String> {
    let found = matching(id)?;
    let next = !found.iter().all(|(_, stream)| stream.muted);
    for (index, _) in found {
        run("pactl", &["set-sink-input-mute", &index.to_string(), if next { "1" } else { "0" }])?;
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test manuel : demande un serveur audio (PulseAudio ou PipeWire) où au moins une application
    /// joue. Lancé dans le conteneur par `packaging/linux/mixer-smoke-test.sh`.
    #[test]
    #[ignore = "dépend d'un serveur audio et d'une application qui joue"]
    fn controls_real_app_streams() {
        let apps = get_app_volumes().expect("liste des applications");
        println!("avant : {apps:?}");
        let app = apps.first().expect("au moins une application qui joue");

        set_app_volume(&app.id, 42).expect("réglage du volume");
        let volume = get_app_volumes().unwrap().into_iter().find(|a| a.id == app.id).unwrap().volume;
        assert_eq!(volume, 42);

        let muted = toggle_app_mute(&app.id).expect("mute");
        let after = get_app_volumes().unwrap().into_iter().find(|a| a.id == app.id).unwrap();
        println!("après : {after:?}");
        assert_eq!(after.muted, muted);
        assert!(muted, "l'application n'était pas muette, elle doit l'être après l'inversion");
    }
}
