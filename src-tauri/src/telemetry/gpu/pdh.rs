//! Compteurs de performance Windows (PDH) pour la charge GPU et la VRAM utilisée.
//! Ces compteurs existent pour toutes les marques de GPU sous Windows 10 et 11.

use std::collections::HashMap;
use std::sync::Mutex;

use windows::core::{w, PCWSTR};
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
    PdhOpenQueryW, PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE_ITEM_W,
    PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA,
};

const BYTES_PER_MB: f64 = 1024.0 * 1024.0;

pub struct GpuSample {
    pub usage: f32,
    pub vram_used_mb: f32,
}

struct Query {
    handle: PDH_HQUERY,
    engines: PDH_HCOUNTER,
    memory: PDH_HCOUNTER,
}

// Les handles PDH sont de simples pointeurs opaques, utilisés ici derrière un Mutex.
unsafe impl Send for Query {}

impl Drop for Query {
    fn drop(&mut self) {
        unsafe { PdhCloseQuery(self.handle) };
    }
}

static QUERY: Mutex<Option<Query>> = Mutex::new(None);

fn open() -> Option<Query> {
    let mut handle = PDH_HQUERY::default();
    if unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut handle) } != 0 {
        return None;
    }

    let mut engines = PDH_HCOUNTER::default();
    let mut memory = PDH_HCOUNTER::default();
    let added = unsafe {
        PdhAddEnglishCounterW(handle, w!("\\GPU Engine(*)\\Utilization Percentage"), 0, &mut engines) == 0
            && PdhAddEnglishCounterW(handle, w!("\\GPU Adapter Memory(*)\\Dedicated Usage"), 0, &mut memory) == 0
    };

    let query = Query { handle, engines, memory };
    // Si un ajout a échoué, `query` est détruit ici et ferme le handle via Drop.
    added.then_some(query)
}

/// Charge et VRAM utilisée de la carte identifiée par `luid`.
/// Retourne `None` pour le premier échantillon : les compteurs de taux ont besoin de deux mesures.
pub fn sample(luid: &str) -> Option<GpuSample> {
    let mut guard = QUERY.lock().ok()?;

    if guard.is_none() {
        let query = open()?;
        // Première mesure de référence, la valeur utile arrive au tick suivant.
        unsafe { PdhCollectQueryData(query.handle) };
        *guard = Some(query);
        return None;
    }

    let query = guard.as_ref()?;
    if unsafe { PdhCollectQueryData(query.handle) } != 0 {
        return None;
    }

    let engines = read_items(query.engines)?;
    let usage = engine_usage(&engines, luid)?;
    let vram_bytes: f64 = read_items(query.memory)
        .map(|items| {
            items
                .iter()
                .filter(|(name, _)| name.to_ascii_lowercase().contains(luid))
                .map(|(_, value)| *value)
                .sum()
        })
        .unwrap_or(0.0);

    Some(GpuSample {
        usage,
        vram_used_mb: (vram_bytes / BYTES_PER_MB) as f32,
    })
}

/// Pour chaque moteur (3D, copie, vidéo...), on additionne la charge de tous les processus,
/// puis on retient le moteur le plus chargé : c'est ce que le Gestionnaire des tâches affiche.
fn engine_usage(items: &[(String, f64)], luid: &str) -> Option<f32> {
    let mut per_engine: HashMap<String, f64> = HashMap::new();

    for (name, value) in items {
        let lower = name.to_ascii_lowercase();
        if !lower.contains(luid) {
            continue;
        }
        // Instance : pid_1234_luid_0x..._0x..._phys_0_eng_0_engtype_3D. On garde la partie moteur.
        let Some((_, engine)) = lower.split_once("_phys_") else {
            continue;
        };
        *per_engine.entry(engine.to_string()).or_insert(0.0) += *value;
    }

    per_engine
        .values()
        .cloned()
        .fold(None, |best: Option<f64>, value| Some(best.map_or(value, |b| b.max(value))))
        .map(|value| value.clamp(0.0, 100.0) as f32)
}

/// Lit toutes les instances d'un compteur générique, au format décimal.
fn read_items(counter: PDH_HCOUNTER) -> Option<Vec<(String, f64)>> {
    let mut size = 0u32;
    let mut count = 0u32;

    let status = unsafe { PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, None) };
    if status != PDH_MORE_DATA {
        return None;
    }

    // Vec<u64> garantit l'alignement sur 8 octets exigé par la structure PDH.
    let mut buffer = vec![0u64; (size as usize).div_ceil(8)];
    let items_ptr = buffer.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;

    let status = unsafe {
        PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, Some(items_ptr))
    };
    if status != 0 {
        return None;
    }

    let items = unsafe { std::slice::from_raw_parts(items_ptr, count as usize) };
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let status = item.FmtValue.CStatus;
        if status != PDH_CSTATUS_VALID_DATA && status != PDH_CSTATUS_NEW_DATA {
            continue;
        }
        let Ok(name) = (unsafe { item.szName.to_string() }) else {
            continue;
        };
        let value = unsafe { item.FmtValue.Anonymous.doubleValue };
        result.push((name, value));
    }
    Some(result)
}
