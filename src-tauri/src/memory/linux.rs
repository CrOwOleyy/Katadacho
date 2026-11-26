//! Nettoyage de la mémoire sous Linux. Libérer les caches du noyau (`drop_caches`) et
//! compacter la mémoire exige root : on passe par `pkexec`, qui affiche la fenêtre
//! d'authentification du bureau. Rien n'est libéré sans cette autorisation.

use std::process::Command;

use super::CleanResult;
use crate::platform::unsupported;

/// Écrit dans `/proc/sys/vm` : synchronise les disques, vide les caches de pages, de dentries et
/// d'inodes, puis demande au noyau de compacter la mémoire (absent sur certains noyaux, ignoré).
const DEEP_SCRIPT: &str = "sync && echo 3 > /proc/sys/vm/drop_caches && { echo 1 > /proc/sys/vm/compact_memory 2>/dev/null || true; }";

/// Code de sortie de `pkexec` quand l'utilisateur ferme la fenêtre ou refuse.
const PKEXEC_DISMISSED: i32 = 126;
const PKEXEC_NOT_AUTHORIZED: i32 = 127;

/// Sans droits administrateur, Linux ne permet de libérer aucune mémoire d'un autre processus.
pub fn quick() -> Result<CleanResult, String> {
    Err(format!(
        "{}. Utilise le nettoyage profond, qui demande ton mot de passe.",
        unsupported("Le nettoyage rapide")
    ))
}

/// Retourne les étapes qui ont échoué (vide quand tout a réussi).
pub fn deep() -> Result<Vec<String>, String> {
    let direct = nix_is_root();
    let output = if direct {
        Command::new("sh").args(["-c", DEEP_SCRIPT]).output()
    } else {
        Command::new("pkexec").args(["sh", "-c", DEEP_SCRIPT]).output()
    }
    .map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            "pkexec est introuvable : installe polkit pour autoriser le nettoyage".to_string()
        } else {
            error.to_string()
        }
    })?;

    match output.status.code() {
        Some(0) => Ok(Vec::new()),
        Some(PKEXEC_DISMISSED) | Some(PKEXEC_NOT_AUTHORIZED) if !direct => {
            Err("Autorisation refusée ou fenêtre fermée".to_string())
        }
        _ => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
    }
}

fn nix_is_root() -> bool {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find_map(|line| line.strip_prefix("Uid:").map(|rest| rest.split_whitespace().nth(1).map(str::to_owned)))
                .flatten()
        })
        .is_some_and(|euid| euid == "0")
}
