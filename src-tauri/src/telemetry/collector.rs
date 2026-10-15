use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use sysinfo::{Components, Disks, Networks, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};

use super::gpu::{self, GpuStats};
use super::processes::{top_groups, ProcessGroup};

pub const EVENT_SYSTEM_STATS: &str = "system-stats";
const TICK: Duration = Duration::from_millis(1000);
const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;
const BYTES_PER_MB: f64 = 1024.0 * 1024.0;

/// Drapeau global : la boucle tourne tant qu'il est à `true`.
pub static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CpuStats {
    pub usage: f32,
    pub frequency_ghz: f32,
    pub temperature_c: Option<f32>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStats {
    pub used_gb: f32,
    pub total_gb: f32,
    pub percent: f32,
    pub swap_used_gb: f32,
    pub swap_total_gb: f32,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiskStats {
    pub used_gb: f32,
    pub total_gb: f32,
    /// Débit de lecture et d'écriture agrégé des processus, en Mo/s.
    pub read_mb_s: f32,
    pub write_mb_s: f32,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStats {
    /// Débit reçu et émis sur toutes les interfaces, en Mo/s.
    pub download_mb_s: f32,
    pub upload_mb_s: f32,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu: CpuStats,
    pub memory: MemoryStats,
    pub disk: DiskStats,
    pub network: NetworkStats,
    pub gpu: Option<GpuStats>,
    pub processes: Vec<ProcessGroup>,
}

/// Démarre la boucle de télémétrie si elle ne tourne pas déjà.
pub fn start(app: AppHandle) {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(move || {
        let mut system = System::new_all();
        let mut disks = Disks::new_with_refreshed_list();
        let mut networks = Networks::new_with_refreshed_list();
        let mut components = Components::new_with_refreshed_list();
        let cpu_count = system.cpus().len();
        let mut last_tick = Instant::now();

        while RUNNING.load(Ordering::SeqCst) {
            let started = Instant::now();
            let elapsed = started.duration_since(last_tick).as_secs_f64().max(0.001);
            last_tick = started;

            system.refresh_cpu_usage();
            system.refresh_memory();
            system.refresh_processes(ProcessesToUpdate::All, true);
            disks.refresh(true);
            networks.refresh(true);
            components.refresh(true);

            let stats = collect(&system, &disks, &networks, &components, cpu_count, elapsed);

            if app.emit(EVENT_SYSTEM_STATS, stats).is_err() {
                RUNNING.store(false, Ordering::SeqCst);
                break;
            }

            let spent = started.elapsed();
            if spent < TICK {
                thread::sleep(TICK - spent);
            }
        }
    });
}

pub fn stop() {
    RUNNING.store(false, Ordering::SeqCst);
}

fn collect(
    system: &System,
    disks: &Disks,
    networks: &Networks,
    components: &Components,
    cpu_count: usize,
    elapsed: f64,
) -> SystemStats {
    let cpu_frequency_mhz = system.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0);
    let temperature = components
        .iter()
        .filter(|component| component.label().to_ascii_lowercase().contains("cpu"))
        .find_map(|component| component.temperature());

    let total_memory = system.total_memory() as f64;
    let used_memory = system.used_memory() as f64;
    let total_swap = system.total_swap() as f64;
    let used_swap = system.used_swap() as f64;

    // Disque principal : celui qui porte la racine Windows, sinon le premier disque.
    let system_disk = disks
        .iter()
        .find(|disk| disk.mount_point().to_string_lossy().starts_with("C:"))
        .or_else(|| disks.iter().next());
    let (disk_used, disk_total) = system_disk
        .map(|disk| {
            let total = disk.total_space() as f64;
            let used = total - disk.available_space() as f64;
            (used, total)
        })
        .unwrap_or((0.0, 0.0));

    // Les compteurs sont des octets écoulés depuis le dernier tick.
    let (read_bytes, written_bytes) = system.processes().values().fold((0u64, 0u64), |acc, process| {
        let usage = process.disk_usage();
        (acc.0 + usage.read_bytes, acc.1 + usage.written_bytes)
    });

    let (received, transmitted) = networks.iter().fold((0u64, 0u64), |acc, (_, data)| {
        (acc.0 + data.received(), acc.1 + data.transmitted())
    });

    SystemStats {
        cpu: CpuStats {
            usage: system.global_cpu_usage(),
            frequency_ghz: cpu_frequency_mhz as f32 / 1000.0,
            temperature_c: temperature,
        },
        memory: MemoryStats {
            used_gb: (used_memory / BYTES_PER_GB) as f32,
            total_gb: (total_memory / BYTES_PER_GB) as f32,
            percent: if total_memory > 0.0 {
                (used_memory / total_memory * 100.0) as f32
            } else {
                0.0
            },
            swap_used_gb: (used_swap / BYTES_PER_GB) as f32,
            swap_total_gb: (total_swap / BYTES_PER_GB) as f32,
        },
        disk: DiskStats {
            used_gb: (disk_used / BYTES_PER_GB) as f32,
            total_gb: (disk_total / BYTES_PER_GB) as f32,
            read_mb_s: (read_bytes as f64 / elapsed / BYTES_PER_MB) as f32,
            write_mb_s: (written_bytes as f64 / elapsed / BYTES_PER_MB) as f32,
        },
        network: NetworkStats {
            download_mb_s: (received as f64 / elapsed / BYTES_PER_MB) as f32,
            upload_mb_s: (transmitted as f64 / elapsed / BYTES_PER_MB) as f32,
        },
        gpu: gpu::read(),
        processes: top_groups(system, cpu_count),
    }
}
