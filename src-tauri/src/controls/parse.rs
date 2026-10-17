//! Analyse de la sortie des outils en ligne de commande utilisés sous Linux (`pactl`, `nmcli`,
//! `bluetoothctl`). Ces fonctions ne lancent rien : elles sont testées sur n'importe quel système.
// Sous Windows ces fonctions ne servent qu'aux tests.
#![cfg_attr(windows, allow(dead_code))]

use serde_json::Value;

use super::types::{AudioDevice, AudioState};

/// Plus grand pourcentage de volume parmi les canaux d'une sortie (`"45%"` donne 45).
fn sink_volume(sink: &Value) -> u8 {
    sink["volume"]
        .as_object()
        .into_iter()
        .flat_map(|channels| channels.values())
        .filter_map(|channel| channel["value_percent"].as_str())
        .filter_map(|text| text.trim().trim_end_matches('%').parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        .min(100) as u8
}

/// Construit l'état audio depuis `pactl --format=json list sinks` et le nom de la sortie par défaut.
pub fn audio_state(sinks_json: &str, default_sink: &str) -> Result<AudioState, String> {
    let sinks: Value = serde_json::from_str(sinks_json).map_err(|e| format!("Sortie de pactl illisible : {e}"))?;
    let list = sinks.as_array().ok_or("Sortie de pactl inattendue")?;

    let devices: Vec<AudioDevice> = list
        .iter()
        .filter_map(|sink| {
            let id = sink["name"].as_str()?.to_string();
            let name = sink["description"].as_str().unwrap_or(&id).to_string();
            Some(AudioDevice { is_default: id == default_sink, id, name })
        })
        .collect();

    let active = list
        .iter()
        .find(|sink| sink["name"].as_str() == Some(default_sink))
        .ok_or("Aucune sortie audio par défaut")?;

    Ok(AudioState {
        volume: sink_volume(active),
        muted: active["mute"].as_bool().unwrap_or(false),
        active_device: devices.iter().find(|device| device.is_default).cloned(),
        devices,
    })
}

/// `Mute: yes` de `pactl get-sink-mute`.
pub fn is_muted(output: &str) -> bool {
    output.to_lowercase().contains("yes")
}

/// Découpe une ligne `nmcli -t` : le séparateur est `:`, un `:` ou un `\` du contenu est précédé d'un `\`.
pub fn split_terse(line: &str) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    fields.last_mut().expect("au moins un champ").push(next);
                }
            }
            ':' => fields.push(String::new()),
            other => fields.last_mut().expect("au moins un champ").push(other),
        }
    }
    fields
}

/// Réseau Wi-Fi actif depuis `nmcli -t -f ACTIVE,SSID,SIGNAL device wifi`.
pub fn active_wifi(output: &str) -> Option<(String, u8)> {
    output.lines().find_map(|line| {
        let fields = split_terse(line);
        if fields.len() < 3 || fields[0] != "yes" || fields[1].is_empty() {
            return None;
        }
        let signal = fields[2].trim().parse::<u32>().ok()?.min(100) as u8;
        Some((fields[1].clone(), signal))
    })
}

/// `enabled` / `disabled` de `nmcli radio wifi`.
pub fn radio_enabled(output: &str) -> Option<bool> {
    match output.trim() {
        "enabled" => Some(true),
        "disabled" => Some(false),
        _ => None,
    }
}

/// `Powered: yes` de `bluetoothctl show`. `None` quand il n'y a aucun contrôleur.
pub fn bluetooth_powered(output: &str) -> Option<bool> {
    output.lines().find_map(|line| {
        let value = line.trim().strip_prefix("Powered:")?;
        Some(value.trim() == "yes")
    })
}

/// Noms des appareils de `bluetoothctl devices Connected` (`Device AA:BB:.. Nom`).
pub fn bluetooth_devices(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("Device ")?;
            let (_address, name) = rest.split_once(' ')?;
            let name = name.trim();
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

/// Pourcentage de luminosité (0 à 100) pour une valeur brute du pilote.
pub fn brightness_percent(raw: u32, max: u32) -> u8 {
    if max == 0 {
        return 0;
    }
    ((raw as u64 * 100 + max as u64 / 2) / max as u64).min(100) as u8
}

/// Valeur brute du pilote pour un pourcentage. Jamais 0 : un écran complètement noir est inutilisable.
pub fn brightness_raw(percent: u8, max: u32) -> u32 {
    let raw = (percent.min(100) as u64 * max as u64 + 50) / 100;
    (raw as u32).max(1).min(max.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINKS: &str = r#"[
      {"name":"alsa_output.hdmi","description":"HDMI","mute":false,
       "volume":{"front-left":{"value":1,"value_percent":"20%"},"front-right":{"value":1,"value_percent":"20%"}}},
      {"name":"alsa_output.analog","description":"Haut-parleurs","mute":true,
       "volume":{"front-left":{"value":1,"value_percent":"45%"},"front-right":{"value":1,"value_percent":"63%"}}}
    ]"#;

    #[test]
    fn audio_state_reads_the_default_sink() {
        let state = audio_state(SINKS, "alsa_output.analog").unwrap();
        assert_eq!(state.volume, 63);
        assert!(state.muted);
        assert_eq!(state.devices.len(), 2);
        assert_eq!(state.active_device.unwrap().name, "Haut-parleurs");
    }

    #[test]
    fn audio_state_fails_without_a_matching_default() {
        assert!(audio_state(SINKS, "absent").is_err());
        assert!(audio_state("pas du json", "x").is_err());
    }

    #[test]
    fn mute_output_is_read() {
        assert!(is_muted("Mute: yes"));
        assert!(!is_muted("Mute: no"));
    }

    #[test]
    fn terse_lines_handle_escaped_colons() {
        assert_eq!(split_terse(r"yes:Cafe\: Wi-Fi:78"), vec!["yes", "Cafe: Wi-Fi", "78"]);
    }

    #[test]
    fn active_wifi_picks_the_connected_network() {
        let output = "no:Voisin:80\nyes:Maison:67\nno:Autre:20\n";
        assert_eq!(active_wifi(output), Some(("Maison".to_string(), 67)));
        assert_eq!(active_wifi("no:Voisin:80\n"), None);
    }

    #[test]
    fn radio_state_is_read() {
        assert_eq!(radio_enabled("enabled\n"), Some(true));
        assert_eq!(radio_enabled("disabled\n"), Some(false));
        assert_eq!(radio_enabled("autre"), None);
    }

    #[test]
    fn bluetooth_power_and_devices_are_read() {
        let show = "Controller AA:BB (public)\n\tName: pc\n\tPowered: yes\n\tDiscovering: no\n";
        assert_eq!(bluetooth_powered(show), Some(true));
        assert_eq!(bluetooth_powered("No default controller available"), None);
        assert_eq!(
            bluetooth_devices("Device 11:22:33:44:55:66 Casque Sony\nDevice AA:BB:CC:DD:EE:FF Souris\n"),
            vec!["Casque Sony", "Souris"]
        );
    }

    #[test]
    fn brightness_conversion_round_trips_and_never_goes_black() {
        assert_eq!(brightness_percent(50, 100), 50);
        assert_eq!(brightness_percent(0, 0), 0);
        assert_eq!(brightness_raw(50, 255), 128);
        assert_eq!(brightness_raw(0, 255), 1);
        assert_eq!(brightness_raw(100, 255), 255);
        assert_eq!(brightness_percent(brightness_raw(37, 400), 400), 37);
    }
}
