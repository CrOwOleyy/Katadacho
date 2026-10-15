//! Informations statiques de la carte graphique via DXGI : nom, VRAM totale et LUID.
//! Fonctionne avec AMD, Intel et NVIDIA sans pilote propriétaire.

use std::sync::OnceLock;

use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
};

const BYTES_PER_MB: f64 = 1024.0 * 1024.0;

pub struct AdapterInfo {
    pub name: String,
    /// Fragment d'instance PDH de cette carte, en minuscules : `luid_0xHHHHHHHH_0xLLLLLLLL`.
    pub luid: String,
    pub vram_total_mb: f32,
}

static ADAPTER: OnceLock<Option<AdapterInfo>> = OnceLock::new();

/// Carte matérielle avec le plus de VRAM dédiée. Les adaptateurs logiciels sont ignorés.
/// Le résultat est calculé une seule fois.
pub fn primary_adapter() -> Option<&'static AdapterInfo> {
    ADAPTER.get_or_init(detect).as_ref()
}

fn detect() -> Option<AdapterInfo> {
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.ok()?;

    let mut best: Option<AdapterInfo> = None;
    let mut best_vram = 0usize;
    let mut index = 0u32;

    while let Ok(adapter) = unsafe { factory.EnumAdapters1(index) } {
        index += 1;
        let Ok(desc) = (unsafe { adapter.GetDesc1() }) else {
            continue;
        };
        if desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 {
            continue;
        }

        // Pour une carte intégrée la VRAM dédiée peut être faible : on garde la première
        // carte rencontrée si toutes sont à zéro.
        if best.is_none() || desc.DedicatedVideoMemory > best_vram {
            best_vram = desc.DedicatedVideoMemory;
            let name_len = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
            best = Some(AdapterInfo {
                name: String::from_utf16_lossy(&desc.Description[..name_len]),
                luid: format!(
                    "luid_0x{:08x}_0x{:08x}",
                    desc.AdapterLuid.HighPart as u32, desc.AdapterLuid.LowPart
                ),
                vram_total_mb: (desc.DedicatedVideoMemory as f64 / BYTES_PER_MB) as f32,
            });
        }
    }

    best
}
