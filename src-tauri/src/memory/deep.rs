//! Nettoyage profond de la mémoire, qui reproduit les quatre étapes de RAMMap (`-Ew`, `-Es`,
//! `-Em`, `-E0`) avec les appels système natifs. Il exige les droits administrateur : si
//! l'application ne les a pas, elle se relance elle-même élevée (fenêtre UAC) pour cette seule tâche.

use std::ffi::c_void;
use std::mem::{size_of, transmute};

use windows::core::{s, w, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_NOT_ALL_ASSIGNED, HANDLE, LUID, WAIT_TIMEOUT};
use windows::Win32::Security::{
    AdjustTokenPrivileges, GetTokenInformation, LookupPrivilegeValueW, TokenElevation, LUID_AND_ATTRIBUTES,
    SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_ELEVATION, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Memory::SetSystemFileCacheSize;
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeProcess, OpenProcessToken, WaitForSingleObject,
};
use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

/// Argument qui fait exécuter ce nettoyage à l'exécutable, au lieu de lancer l'application.
pub const CLI_FLAG: &str = "--clean-memory";

const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 80;
const MEMORY_EMPTY_WORKING_SETS: u32 = 2;
const MEMORY_FLUSH_MODIFIED_LIST: u32 = 3;
const MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST: u32 = 5;
const ERROR_CANCELLED: u32 = 1223;
/// Au-delà, on considère que le nettoyage élevé est bloqué.
const ELEVATED_TIMEOUT_MS: u32 = 120_000;

/// Codes de sortie du processus élevé.
const EXIT_OK: i32 = 0;
const EXIT_NOT_ELEVATED: i32 = 1;
const EXIT_PARTIAL: i32 = 3;

type NtSetSystemInformation = unsafe extern "system" fn(u32, *mut c_void, u32) -> i32;

/// `NtSetSystemInformation` n'est pas exposée par la crate `windows` : on la charge dans ntdll.
fn nt_set_system_information() -> Result<NtSetSystemInformation, String> {
    let module = unsafe { GetModuleHandleW(w!("ntdll.dll")) }.map_err(|e| e.to_string())?;
    let function = unsafe { GetProcAddress(module, s!("NtSetSystemInformation")) }
        .ok_or_else(|| "NtSetSystemInformation introuvable".to_string())?;
    Ok(unsafe { transmute::<_, NtSetSystemInformation>(function) })
}

pub fn is_elevated() -> bool {
    let mut token = HANDLE::default();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }.is_err() {
        return false;
    }
    let mut elevation = TOKEN_ELEVATION::default();
    let mut returned = 0u32;
    let queried = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut TOKEN_ELEVATION as *mut c_void),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
    };
    let _ = unsafe { CloseHandle(token) };
    queried.is_ok() && elevation.TokenIsElevated != 0
}

fn enable_privilege(name: PCWSTR) -> Result<(), String> {
    let mut token = HANDLE::default();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) }
        .map_err(|e| e.to_string())?;

    let result = (|| {
        let mut luid = LUID::default();
        unsafe { LookupPrivilegeValueW(None, name, &mut luid) }.map_err(|e| e.to_string())?;
        let privileges = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES { Luid: luid, Attributes: SE_PRIVILEGE_ENABLED }],
        };
        unsafe { AdjustTokenPrivileges(token, false, Some(&privileges), 0, None, None) }.map_err(|e| e.to_string())?;
        // AdjustTokenPrivileges réussit même si le privilège n'est pas détenu : il faut lire l'erreur.
        if unsafe { GetLastError() } == ERROR_NOT_ALL_ASSIGNED {
            return Err("Privilège non accordé à ce compte".to_string());
        }
        Ok(())
    })();

    let _ = unsafe { CloseHandle(token) };
    result
}

fn memory_list_command(command: u32) -> Result<(), String> {
    let function = nt_set_system_information()?;
    let mut value = command;
    let status = unsafe { function(SYSTEM_MEMORY_LIST_INFORMATION, &mut value as *mut u32 as *mut c_void, 4) };
    if status < 0 {
        return Err(format!("NTSTATUS 0x{:08X}", status as u32));
    }
    Ok(())
}

/// Les quatre étapes, dans l'ordre du script d'origine. Retourne le nom des étapes qui ont échoué.
fn run_steps() -> Vec<String> {
    let steps: [(&str, fn() -> Result<(), String>); 4] = [
        ("Espaces de travail des processus", || memory_list_command(MEMORY_EMPTY_WORKING_SETS)),
        ("Cache de fichiers système", || {
            unsafe { SetSystemFileCacheSize(usize::MAX, usize::MAX, 0) }.map_err(|e| e.to_string())
        }),
        ("Liste des pages modifiées", || memory_list_command(MEMORY_FLUSH_MODIFIED_LIST)),
        ("Liste de veille de priorité 0", || memory_list_command(MEMORY_PURGE_LOW_PRIORITY_STANDBY_LIST)),
    ];

    steps
        .iter()
        .filter_map(|(label, step)| step().err().map(|error| format!("{label} ({error})")))
        .collect()
}

/// Exécute le nettoyage dans le processus courant, qui doit être élevé.
/// Retourne les étapes échouées, ou une erreur si les droits manquent.
pub fn run_in_process() -> Result<Vec<String>, String> {
    if !is_elevated() {
        return Err("Droits administrateur requis".to_string());
    }
    enable_privilege(w!("SeProfileSingleProcessPrivilege"))?;
    enable_privilege(w!("SeIncreaseQuotaPrivilege"))?;
    Ok(run_steps())
}

/// Point d'entrée du processus élevé (`katadacho.exe --clean-memory`). Retourne le code de sortie.
pub fn run_cli() -> i32 {
    match run_in_process() {
        Ok(failed) if failed.is_empty() => EXIT_OK,
        Ok(_) => EXIT_PARTIAL,
        Err(_) => EXIT_NOT_ELEVATED,
    }
}

/// Relance l'exécutable avec les droits administrateur (fenêtre UAC) et attend la fin du nettoyage.
/// Retourne les étapes échouées : vide si tout est passé.
pub fn run_elevated() -> Result<Vec<String>, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut exe_wide: Vec<u16> = exe.as_os_str().encode_wide_with_nul();
    let parameters: Vec<u16> = CLI_FLAG.encode_utf16().chain(std::iter::once(0)).collect();

    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
        lpVerb: w!("runas"),
        lpFile: PCWSTR(exe_wide.as_mut_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };

    if let Err(error) = unsafe { ShellExecuteExW(&mut info) } {
        // L'utilisateur a refusé la fenêtre UAC : ce n'est pas une panne.
        return Err(if error.code().0 as u32 & 0xFFFF == ERROR_CANCELLED {
            "Autorisation administrateur refusée".to_string()
        } else {
            error.to_string()
        });
    }

    let process = info.hProcess;
    let waited = unsafe { WaitForSingleObject(process, ELEVATED_TIMEOUT_MS) };
    if waited == WAIT_TIMEOUT {
        let _ = unsafe { CloseHandle(process) };
        return Err("Le nettoyage administrateur ne répond pas".to_string());
    }

    let mut code = 0u32;
    let got_code = unsafe { GetExitCodeProcess(process, &mut code) };
    let _ = unsafe { CloseHandle(process) };
    got_code.map_err(|e| e.to_string())?;

    match code as i32 {
        EXIT_OK => Ok(Vec::new()),
        EXIT_PARTIAL => Ok(vec!["Une partie du nettoyage a échoué".to_string()]),
        EXIT_NOT_ELEVATED => Err("Les droits administrateur n'ont pas été accordés".to_string()),
        other => Err(format!("Le nettoyage s'est terminé avec le code {other}")),
    }
}

/// Petit utilitaire : texte en UTF-16 terminé par un zéro, pour les API Windows.
trait EncodeWideWithNul {
    fn encode_wide_with_nul(&self) -> Vec<u16>;
}

impl EncodeWideWithNul for std::ffi::OsStr {
    fn encode_wide_with_nul(&self) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        self.encode_wide().chain(std::iter::once(0)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_elevation_state_without_panicking() {
        // Le résultat dépend de la session : on vérifie seulement que l'appel aboutit.
        let _ = is_elevated();
    }

    #[test]
    fn refuses_to_run_without_elevation() {
        if !is_elevated() {
            assert_eq!(run_in_process(), Err("Droits administrateur requis".to_string()));
        }
    }

    #[test]
    fn nt_function_is_resolvable() {
        assert!(nt_set_system_information().is_ok());
    }
}
