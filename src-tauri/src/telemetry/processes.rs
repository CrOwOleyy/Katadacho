use std::collections::HashMap;

use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};

const TOP_COUNT: usize = 5;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProcessGroup {
    /// Nom de l'application, sous-processus regroupés (ex. chrome.exe).
    pub name: String,
    /// PID du sous-processus le plus gourmand du groupe, cible du bouton « Kill ».
    pub pid: u32,
    /// Charge CPU totale du groupe, en % de l'ensemble des cœurs.
    pub cpu: f32,
    /// Mémoire totale du groupe, en Mo.
    pub memory_mb: f32,
    /// Nombre de sous-processus regroupés.
    pub count: usize,
}

struct Accumulator {
    name: String,
    best_pid: Pid,
    best_cpu: f32,
    cpu: f32,
    memory: u64,
    count: usize,
}

/// Regroupe les processus par nom et retourne les `TOP_COUNT` plus gourmands.
/// Le tri se fait sur le CPU, puis sur la mémoire pour départager.
pub fn top_groups(system: &System, cpu_count: usize) -> Vec<ProcessGroup> {
    let mut groups: HashMap<String, Accumulator> = HashMap::new();
    let cores = cpu_count.max(1) as f32;

    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy().into_owned();
        let cpu = process.cpu_usage() / cores;
        let memory = process.memory();

        let entry = groups.entry(name.clone()).or_insert(Accumulator {
            name,
            best_pid: *pid,
            best_cpu: -1.0,
            cpu: 0.0,
            memory: 0,
            count: 0,
        });

        entry.cpu += cpu;
        entry.memory += memory;
        entry.count += 1;
        if cpu > entry.best_cpu {
            entry.best_cpu = cpu;
            entry.best_pid = *pid;
        }
    }

    let mut list: Vec<Accumulator> = groups.into_values().collect();
    list.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.memory.cmp(&a.memory))
    });

    list.into_iter()
        .take(TOP_COUNT)
        .map(|group| ProcessGroup {
            name: group.name,
            pid: group.best_pid.as_u32(),
            cpu: group.cpu,
            memory_mb: group.memory as f32 / (1024.0 * 1024.0),
            count: group.count,
        })
        .collect()
}

/// Termine le processus donné. Retourne une erreur si le PID n'existe plus.
pub fn kill(pid: u32) -> Result<(), String> {
    let target = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[target]), true);

    let process = system
        .process(target)
        .ok_or_else(|| format!("Le processus {pid} n'existe plus"))?;

    if process.kill() {
        Ok(())
    } else {
        Err(format!("Impossible de terminer le processus {pid}"))
    }
}
