#[cfg(windows)]
mod dxgi;
mod nvml;
#[cfg(windows)]
mod pdh;
#[cfg(target_os = "linux")]
mod sysfs;

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GpuStats {
    pub name: String,
    /// Charge du cœur GPU, en %.
    pub usage: f32,
    pub vram_used_mb: f32,
    pub vram_total_mb: f32,
}

/// Lit les stats du GPU : NVML pour NVIDIA sur tous les systèmes, sinon le repli du système :
/// DXGI et PDH sous Windows (toutes marques), les fichiers du pilote `amdgpu` sous Linux.
pub fn read() -> Option<GpuStats> {
    nvml::read().or_else(read_platform)
}

#[cfg(target_os = "linux")]
fn read_platform() -> Option<GpuStats> {
    sysfs::read()
}

/// Pas encore de repli universel hors Windows et Linux : seul NVML est utilisé.
#[cfg(not(any(windows, target_os = "linux")))]
fn read_platform() -> Option<GpuStats> {
    None
}

#[cfg(windows)]
fn read_platform() -> Option<GpuStats> {
    let adapter = dxgi::primary_adapter()?;
    let sample = pdh::sample(&adapter.luid)?;

    Some(GpuStats {
        name: adapter.name.clone(),
        usage: sample.usage,
        // Sur une carte intégrée la VRAM dédiée est parfois sous-estimée : on ne dépasse jamais le total.
        vram_used_mb: if adapter.vram_total_mb > 0.0 {
            sample.vram_used_mb.min(adapter.vram_total_mb)
        } else {
            sample.vram_used_mb
        },
        vram_total_mb: adapter.vram_total_mb,
    })
}
