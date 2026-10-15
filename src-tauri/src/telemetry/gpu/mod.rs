mod nvml;

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

/// Lit les stats du GPU. NVML pour NVIDIA, avec repli universel Windows ajouté ensuite.
pub fn read() -> Option<GpuStats> {
    nvml::read()
}
