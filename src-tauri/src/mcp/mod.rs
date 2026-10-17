//! Pont avec Claude Desktop, sans clé d'API, par le protocole MCP :
//! Claude Desktop lance `katadacho.exe --mcp-server`, qui lui expose deux outils et les transmet à
//! l'application déjà ouverte par un canal local authentifié.

pub mod bridge;
pub mod install;
pub mod protocol;
pub mod server;

use std::path::PathBuf;

use tauri::State;

use bridge::PendingQuestions;
use install::ConfigStatus;

/// Argument qui fait exécuter l'exécutable en serveur MCP au lieu de lancer l'application.
pub const CLI_FLAG: &str = install::CLI_FLAG;

/// Dossier de données de Katadacho, le même que celui de l'application (`state::AppPaths`),
/// retrouvé sans Tauri pour le processus serveur MCP.
pub fn data_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"));
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));

    base.map(|base| base.join("Katadacho"))
}

/// Point d'entrée du processus serveur MCP. Retourne le code de sortie.
pub fn run_cli() -> i32 {
    match data_dir() {
        Some(dir) => server::run(dir),
        None => {
            eprintln!("Dossier de données introuvable");
            1
        }
    }
}

fn current_executable() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| e.to_string())
}

/// État de la connexion à Claude Desktop, un élément par fichier de configuration connu.
#[tauri::command]
pub fn get_claude_bridge_status() -> Result<Vec<ConfigStatus>, String> {
    Ok(install::status(&current_executable()?))
}

/// Ajoute Katadacho aux serveurs MCP de Claude Desktop. Il faut ensuite redémarrer Claude Desktop.
#[tauri::command]
pub fn install_claude_mcp_bridge() -> Result<Vec<String>, String> {
    install::install(&current_executable()?)
}

#[tauri::command]
pub fn uninstall_claude_mcp_bridge() -> Result<Vec<String>, String> {
    install::uninstall()
}

/// Réponse de l'utilisateur à une question posée par Claude. `false` si la question a expiré.
#[tauri::command]
pub fn answer_ai_question(pending: State<'_, PendingQuestions>, id: u64, answer: String) -> bool {
    pending.answer(id, answer)
}
