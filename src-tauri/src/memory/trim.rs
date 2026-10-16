//! Nettoyage rapide, sans droits administrateur : réduit la mémoire de travail de chaque
//! processus accessible (`EmptyWorkingSet`). La mémoire est rendue à Windows, qui la rend
//! de nouveau au processus si celui-ci en a besoin.

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::ProcessStatus::{EmptyWorkingSet, EnumProcesses};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA};

/// Capacité de la liste de PID. Au-delà, les processus en trop sont simplement ignorés.
const MAX_PROCESSES: usize = 8192;
/// Les PID 0 et 4 sont le système lui-même : inaccessibles.
const FIRST_USER_PID: u32 = 8;

fn process_ids() -> Vec<u32> {
    let mut pids = vec![0u32; MAX_PROCESSES];
    let mut bytes_returned = 0u32;
    let capacity = (pids.len() * std::mem::size_of::<u32>()) as u32;

    if unsafe { EnumProcesses(pids.as_mut_ptr(), capacity, &mut bytes_returned) }.is_err() {
        return Vec::new();
    }
    pids.truncate(bytes_returned as usize / std::mem::size_of::<u32>());
    pids
}

/// Réduit la mémoire de travail de tous les processus accessibles et retourne leur nombre.
/// Les processus protégés ou appartenant à un autre utilisateur refusent et sont ignorés.
pub fn trim_all() -> u32 {
    let mut trimmed = 0u32;

    for pid in process_ids() {
        if pid < FIRST_USER_PID {
            continue;
        }
        let Ok(handle) = (unsafe { OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, false, pid) }) else {
            continue;
        };
        if unsafe { EmptyWorkingSet(handle) }.is_ok() {
            trimmed += 1;
        }
        let _ = unsafe { CloseHandle(handle) };
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_running_processes() {
        assert!(process_ids().len() > 10, "une session Windows a toujours plus de 10 processus");
    }

    #[test]
    fn trims_at_least_our_own_processes() {
        assert!(trim_all() > 0);
    }
}
