use std::sync::OnceLock;

use nvml_wrapper::Nvml;
use serde::Serialize;

const BYTES_PER_MB: f64 = 1024.0 * 1024.0;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GpuStats {
    pub name: String,
    /// Charge du cœur GPU, en %.
    pub usage: f32,
    pub vram_used_mb: f32,
    pub vram_total_mb: f32,
}

/// Initialisation NVML unique. `None` si le pilote NVIDIA est absent, ce qui arrive
/// sur les machines AMD et Intel : on ne réessaie alors pas à chaque tick.
static NVML: OnceLock<Option<Nvml>> = OnceLock::new();

/// Lit l'utilisation et la VRAM de la première carte NVIDIA.
/// Retourne `None` si NVML n'est pas disponible ou si la lecture échoue.
pub fn read_nvidia() -> Option<GpuStats> {
    let nvml = NVML.get_or_init(|| Nvml::init().ok()).as_ref()?;
    let device = nvml.device_by_index(0).ok()?;
    let utilization = device.utilization_rates().ok()?;
    let memory = device.memory_info().ok()?;
    let name = device.name().unwrap_or_else(|_| "GPU NVIDIA".to_string());

    Some(GpuStats {
        name,
        usage: utilization.gpu as f32,
        vram_used_mb: (memory.used as f64 / BYTES_PER_MB) as f32,
        vram_total_mb: (memory.total as f64 / BYTES_PER_MB) as f32,
    })
}
