#[cfg(windows)]
pub mod deep;
#[cfg(not(windows))]
mod linux;
#[cfg(windows)]
pub mod trim;

use std::thread::sleep;
use std::time::Duration;

use serde::Serialize;
use sysinfo::System;

/// Laisse au système le temps de comptabiliser la mémoire rendue avant de la mesurer.
const SETTLE_DELAY: Duration = Duration::from_millis(800);

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    /// `quick` (sans droits administrateur) ou `deep` (administrateur).
    pub mode: &'static str,
    /// Processus dont la mémoire de travail a été réduite. 0 pour le nettoyage profond.
    pub processes_trimmed: u32,
    /// Mémoire rendue, en octets. Peut être négative si Windows a réutilisé la mémoire entre-temps.
    pub freed_bytes: i64,
    pub used_before_bytes: u64,
    pub used_after_bytes: u64,
    pub failed_steps: Vec<String>,
}

fn used_memory() -> u64 {
    let mut system = System::new();
    system.refresh_memory();
    system.used_memory()
}

fn result(mode: &'static str, trimmed: u32, before: u64, after: u64, failed: Vec<String>) -> CleanResult {
    CleanResult {
        mode,
        processes_trimmed: trimmed,
        freed_bytes: before as i64 - after as i64,
        used_before_bytes: before,
        used_after_bytes: after,
        failed_steps: failed,
    }
}

/// Nettoyage rapide, version synchrone : sans droits administrateur.
#[cfg(windows)]
pub fn quick_clean() -> CleanResult {
    let before = used_memory();
    let trimmed = trim::trim_all();
    sleep(SETTLE_DELAY);
    result("quick", trimmed, before, used_memory(), Vec::new())
}

/// Nettoyage rapide. Windows : réduit la mémoire de travail de chaque processus accessible.
/// Linux : rien à libérer sans droits administrateur, l'erreur oriente vers le nettoyage profond.
#[tauri::command]
pub async fn clean_memory_quick() -> Result<CleanResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(windows)]
        {
            Ok(quick_clean())
        }
        #[cfg(not(windows))]
        {
            linux::quick()
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Nettoyage profond. Windows : les quatre étapes de RAMMap, avec une fenêtre UAC si besoin.
/// Linux : vide les caches du noyau, avec une fenêtre d'authentification (polkit) si besoin.
#[tauri::command]
pub async fn clean_memory_deep() -> Result<CleanResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let before = used_memory();
        #[cfg(windows)]
        let failed = if deep::is_elevated() {
            deep::run_in_process()?
        } else {
            deep::run_elevated()?
        };
        #[cfg(not(windows))]
        let failed = linux::deep()?;
        sleep(SETTLE_DELAY);
        Ok(result("deep", 0, before, used_memory(), failed))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    /// Mesure réelle sur la machine : `cargo test --lib quick_clean -- --ignored --nocapture`.
    #[test]
    #[ignore = "dépend de l'état mémoire de la machine"]
    fn quick_clean_reports_a_measurement() {
        let outcome = quick_clean();
        println!(
            "{} processus réduits, avant {:.2} Go, après {:.2} Go, libéré {:.2} Go",
            outcome.processes_trimmed,
            outcome.used_before_bytes as f64 / 1e9,
            outcome.used_after_bytes as f64 / 1e9,
            outcome.freed_bytes as f64 / 1e9,
        );
        assert!(outcome.processes_trimmed > 0);
    }
}
