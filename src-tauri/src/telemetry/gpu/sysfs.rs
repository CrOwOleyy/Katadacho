//! GPU AMD sous Linux, lu dans les fichiers exposés par le pilote `amdgpu` :
//! `/sys/class/drm/cardN/device/gpu_busy_percent`, `mem_info_vram_used` et `mem_info_vram_total`.

use std::fs;
use std::path::{Path, PathBuf};

use super::GpuStats;

const DRM_ROOT: &str = "/sys/class/drm";
const BYTES_PER_MB: f64 = 1024.0 * 1024.0;
const AMD_VENDOR_ID: &str = "0x1002";

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|value| value.trim().to_string())
}

fn read_number(path: &Path) -> Option<f64> {
    read_trimmed(path)?.parse().ok()
}

/// Dossiers `device` des cartes graphiques (`card0`, `card1`…), sans les connecteurs (`card0-HDMI-A-1`).
fn card_devices() -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(DRM_ROOT) else {
        return Vec::new();
    };
    let mut devices: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with("card") && !name.contains('-')
        })
        .map(|entry| entry.path().join("device"))
        .collect();
    devices.sort();
    devices
}

/// Carte AMD avec le plus de VRAM, celle qui affiche en général le bureau et les jeux.
pub fn read() -> Option<GpuStats> {
    card_devices()
        .into_iter()
        .filter(|device| read_trimmed(&device.join("vendor")).as_deref() == Some(AMD_VENDOR_ID))
        .filter_map(|device| {
            let usage = read_number(&device.join("gpu_busy_percent"))?;
            let used = read_number(&device.join("mem_info_vram_used")).unwrap_or(0.0);
            let total = read_number(&device.join("mem_info_vram_total")).unwrap_or(0.0);
            let name = read_trimmed(&device.join("product_name"))
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "GPU AMD".to_string());
            Some(GpuStats {
                name,
                usage: usage.clamp(0.0, 100.0) as f32,
                vram_used_mb: (used / BYTES_PER_MB) as f32,
                vram_total_mb: (total / BYTES_PER_MB) as f32,
            })
        })
        .max_by(|a, b| a.vram_total_mb.total_cmp(&b.vram_total_mb))
}
