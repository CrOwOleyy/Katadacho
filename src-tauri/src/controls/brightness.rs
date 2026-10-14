use serde::{de::IgnoredAny, Deserialize, Serialize};
use windows::core::BOOL;
use windows::Win32::Devices::Display::{
    DestroyPhysicalMonitors, GetMonitorBrightness, GetNumberOfPhysicalMonitorsFromHMONITOR,
    GetPhysicalMonitorsFromHMONITOR, SetMonitorBrightness, PHYSICAL_MONITOR,
};
use windows::Win32::Foundation::{HANDLE, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};
use wmi::WMIConnection;

/// Écrans de portable : classe WMI exposée par le pilote d'affichage intégré.
#[derive(Deserialize)]
#[allow(non_snake_case)]
struct WmiMonitorBrightness {
    CurrentBrightness: u8,
    InstanceName: String,
}

/// Classe des méthodes de luminosité, utilisée pour `WmiSetBrightness`.
/// Le champ n'est jamais lu : le nom de la structure sert de nom de classe WMI.
#[derive(Deserialize)]
#[allow(non_snake_case, dead_code)]
struct WmiMonitorBrightnessMethods {
    InstanceName: String,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct WmiSetBrightnessIn {
    Timeout: u32,
    Brightness: u8,
}

/// Résultat d'un écran DDC/CI : valeurs brutes min et max, nécessaires pour
/// convertir un pourcentage en valeur matérielle.
struct DdcRange {
    min: u32,
    current: u32,
    max: u32,
}

fn wmi_connection() -> Result<WMIConnection, String> {
    WMIConnection::with_namespace_path("ROOT\\WMI").map_err(|e| e.to_string())
}

fn wmi_instances() -> Result<Vec<WmiMonitorBrightness>, String> {
    let con = wmi_connection()?;
    con.query::<WmiMonitorBrightness>().map_err(|e| e.to_string())
}

fn wmi_set(instance_name: &str, level: u8) -> Result<(), String> {
    let con = wmi_connection()?;
    // Les antislashs doivent être doublés dans un chemin d'objet WMI.
    let escaped = instance_name.replace('\\', "\\\\");
    let path = format!("WmiMonitorBrightnessMethods.InstanceName=\"{escaped}\"");
    let params = WmiSetBrightnessIn {
        Timeout: 1,
        Brightness: level,
    };
    con.exec_instance_method::<WmiMonitorBrightnessMethods, IgnoredAny>(path, "WmiSetBrightness", params)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

unsafe extern "system" fn collect_monitor(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    unsafe {
        let list = &mut *(data.0 as *mut Vec<HMONITOR>);
        list.push(monitor);
    }
    BOOL(1)
}

fn display_monitors() -> Vec<HMONITOR> {
    let mut list: Vec<HMONITOR> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(collect_monitor),
            LPARAM(&mut list as *mut Vec<HMONITOR> as isize),
        );
    }
    list
}

/// Ouvre tous les écrans physiques DDC/CI de chaque moniteur, appelle `action`
/// pour chacun, puis libère les handles, même en cas d'erreur.
fn for_each_ddc_monitor<F>(mut action: F)
where
    F: FnMut(HANDLE),
{
    for hmonitor in display_monitors() {
        let mut count: u32 = 0;
        if unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut count) }.is_err() || count == 0 {
            continue;
        }

        let mut physical = vec![PHYSICAL_MONITOR::default(); count as usize];
        if unsafe { GetPhysicalMonitorsFromHMONITOR(hmonitor, &mut physical) }.is_err() {
            continue;
        }

        for monitor in &physical {
            action(monitor.hPhysicalMonitor);
        }
        let _ = unsafe { DestroyPhysicalMonitors(&physical) };
    }
}

fn ddc_range(handle: HANDLE) -> Option<DdcRange> {
    let (mut min, mut current, mut max) = (0u32, 0u32, 0u32);
    let ok = unsafe { GetMonitorBrightness(handle, &mut min, &mut current, &mut max) };
    (ok != 0 && max > min).then_some(DdcRange { min, current, max })
}

fn percent_from_range(range: &DdcRange) -> u8 {
    let span = (range.max - range.min) as f32;
    let value = (range.current - range.min) as f32;
    (value / span * 100.0).round().clamp(0.0, 100.0) as u8
}

fn raw_from_percent(range: &DdcRange, level: u8) -> u32 {
    let span = (range.max - range.min) as f32;
    range.min + (span * f32::from(level.min(100)) / 100.0).round() as u32
}

/// Luminosité actuelle en pourcentage. Priorité à l'écran intégré, puis au premier écran DDC/CI.
pub fn get_brightness() -> Result<u8, String> {
    if let Ok(instances) = wmi_instances() {
        if let Some(first) = instances.first() {
            return Ok(first.CurrentBrightness);
        }
    }

    let mut found: Option<u8> = None;
    for_each_ddc_monitor(|handle| {
        if found.is_none() {
            found = ddc_range(handle).as_ref().map(percent_from_range);
        }
    });
    found.ok_or_else(|| "Aucun écran à luminosité réglable détecté".to_string())
}

/// Règle la luminosité sur tous les écrans détectés (portable et DDC/CI).
pub fn set_brightness(level: u8) -> Result<(), String> {
    let level = level.min(100);
    let mut applied = 0usize;
    let mut last_error: Option<String> = None;

    if let Ok(instances) = wmi_instances() {
        for instance in &instances {
            match wmi_set(&instance.InstanceName, level) {
                Ok(()) => applied += 1,
                Err(e) => last_error = Some(e),
            }
        }
    }

    for_each_ddc_monitor(|handle| {
        if let Some(range) = ddc_range(handle) {
            let raw = raw_from_percent(&range, level);
            if unsafe { SetMonitorBrightness(handle, raw) } != 0 {
                applied += 1;
            }
        }
    });

    if applied > 0 {
        Ok(())
    } else {
        Err(last_error.unwrap_or_else(|| "Aucun écran à luminosité réglable détecté".to_string()))
    }
}
