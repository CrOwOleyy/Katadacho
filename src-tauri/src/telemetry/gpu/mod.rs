mod dxgi;
mod nvml;
mod pdh;

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

/// Lit les stats du GPU : NVML pour NVIDIA, sinon repli universel DXGI et PDH
/// qui couvre AMD, Intel et NVIDIA sans bibliothèque propriétaire.
pub fn read() -> Option<GpuStats> {
    nvml::read().or_else(read_windows)
}

fn read_windows() -> Option<GpuStats> {
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
