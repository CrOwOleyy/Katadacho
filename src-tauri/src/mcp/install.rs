//! Branche Katadacho sur Claude Desktop en ajoutant une entrée `mcpServers` à son fichier
//! `claude_desktop_config.json`. Seule cette entrée est touchée : le reste de la configuration,
//! y compris les autres serveurs MCP, est conservé tel quel.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{json, Map, Value};

/// Nom de l'entrée dans `mcpServers`.
pub const SERVER_NAME: &str = "island-soul-bridge";
/// Argument qui fait exécuter l'exécutable en serveur MCP au lieu de lancer l'application.
pub const CLI_FLAG: &str = "--mcp-server";

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigStatus {
    pub path: String,
    /// Le fichier existe déjà.
    pub exists: bool,
    /// L'entrée Katadacho est présente.
    pub installed: bool,
    /// L'entrée pointe vers l'exécutable actuel (sinon elle est périmée, par exemple après un déplacement).
    pub up_to_date: bool,
}

/// Dossiers où Claude Desktop peut lire sa configuration : l'emplacement classique, et celui du
/// paquet Microsoft Store (`Claude_*`) quand l'application est installée par ce biais.
pub fn config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(appdata) = std::env::var_os("APPDATA") {
        paths.push(PathBuf::from(appdata).join("Claude").join("claude_desktop_config.json"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let packages = PathBuf::from(local).join("Packages");
        if let Ok(read) = fs::read_dir(packages) {
            for entry in read.flatten() {
                if entry.file_name().to_string_lossy().starts_with("Claude_") {
                    paths.push(entry.path().join("LocalCache/Roaming/Claude/claude_desktop_config.json"));
                }
            }
        }
    }
    paths
}

fn server_entry(executable: &Path) -> Value {
    json!({ "command": executable.to_string_lossy(), "args": [CLI_FLAG] })
}

/// Ajoute ou remplace l'entrée Katadacho. Retourne une erreur si le fichier n'est pas un objet JSON,
/// pour ne jamais l'écraser par erreur.
pub fn with_server(config: Value, executable: &Path) -> Result<Value, String> {
    let mut root = match config {
        Value::Object(map) => map,
        Value::Null => Map::new(),
        _ => return Err("La configuration de Claude Desktop n'est pas un objet JSON".to_string()),
    };

    let servers = root.entry("mcpServers").or_insert_with(|| json!({}));
    let Value::Object(servers) = servers else {
        return Err("Le champ « mcpServers » de Claude Desktop n'est pas un objet".to_string());
    };
    servers.insert(SERVER_NAME.to_string(), server_entry(executable));
    Ok(Value::Object(root))
}

/// Retire l'entrée Katadacho. Supprime `mcpServers` s'il devient vide et qu'on l'avait créé.
pub fn without_server(config: Value) -> Result<Value, String> {
    let Value::Object(mut root) = config else {
        return Ok(config);
    };
    if let Some(Value::Object(servers)) = root.get_mut("mcpServers") {
        servers.remove(SERVER_NAME);
        if servers.is_empty() {
            root.remove("mcpServers");
        }
    }
    Ok(Value::Object(root))
}

pub fn inspect(config: &Value, executable: &Path) -> (bool, bool) {
    let entry = config.pointer(&format!("/mcpServers/{SERVER_NAME}"));
    let installed = entry.is_some();
    let up_to_date = entry == Some(&server_entry(executable));
    (installed, up_to_date)
}

fn read_config(path: &Path) -> Result<Value, String> {
    match fs::read_to_string(path) {
        Ok(text) if text.trim().is_empty() => Ok(Value::Null),
        Ok(text) => serde_json::from_str(&text).map_err(|e| format!("JSON invalide dans {} : {e}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Value::Null),
        Err(error) => Err(error.to_string()),
    }
}

/// Écrit la configuration après avoir gardé une copie de l'original, une seule fois.
fn write_config(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let backup = path.with_file_name("claude_desktop_config.json.katadacho-backup");
    if path.exists() && !backup.exists() {
        fs::copy(path, &backup).map_err(|e| format!("Copie de sauvegarde impossible : {e}"))?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

pub fn status(executable: &Path) -> Vec<ConfigStatus> {
    config_paths()
        .into_iter()
        .map(|path| {
            let exists = path.exists();
            let (installed, up_to_date) = read_config(&path)
                .map(|config| inspect(&config, executable))
                .unwrap_or((false, false));
            ConfigStatus { path: path.to_string_lossy().into_owned(), exists, installed, up_to_date }
        })
        .collect()
}

/// Installe l'entrée dans chaque fichier de configuration existant, ou crée le fichier classique
/// si aucun n'existe. Retourne les fichiers modifiés.
pub fn install(executable: &Path) -> Result<Vec<String>, String> {
    let all = config_paths();
    let existing: Vec<PathBuf> = all.iter().filter(|path| path.exists()).cloned().collect();
    let targets = if existing.is_empty() { all.into_iter().take(1).collect() } else { existing };
    if targets.is_empty() {
        return Err("Dossier de configuration de Claude Desktop introuvable".to_string());
    }

    let mut written = Vec::new();
    for path in targets {
        let updated = with_server(read_config(&path)?, executable)?;
        write_config(&path, &updated)?;
        written.push(path.to_string_lossy().into_owned());
    }
    Ok(written)
}

pub fn uninstall() -> Result<Vec<String>, String> {
    let mut written = Vec::new();
    for path in config_paths().into_iter().filter(|path| path.exists()) {
        let config = read_config(&path)?;
        if config.pointer(&format!("/mcpServers/{SERVER_NAME}")).is_none() {
            continue;
        }
        write_config(&path, &without_server(config)?)?;
        written.push(path.to_string_lossy().into_owned());
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXE: &str = r"C:\Apps\Katadacho\katadacho.exe";

    #[test]
    fn adds_the_entry_and_keeps_everything_else() {
        let config = json!({
            "preferences": { "theme": "dark" },
            "mcpServers": { "filesystem": { "command": "npx", "args": ["-y", "fs"] } }
        });
        let updated = with_server(config, Path::new(EXE)).unwrap();

        assert_eq!(updated["preferences"]["theme"], "dark");
        assert_eq!(updated["mcpServers"]["filesystem"]["command"], "npx");
        assert_eq!(updated["mcpServers"][SERVER_NAME]["command"], EXE);
        assert_eq!(updated["mcpServers"][SERVER_NAME]["args"][0], CLI_FLAG);
    }

    #[test]
    fn creates_mcp_servers_when_absent_or_file_empty() {
        let updated = with_server(json!({ "preferences": {} }), Path::new(EXE)).unwrap();
        assert!(updated["mcpServers"][SERVER_NAME].is_object());
        let from_empty = with_server(Value::Null, Path::new(EXE)).unwrap();
        assert!(from_empty["mcpServers"][SERVER_NAME].is_object());
    }

    #[test]
    fn refuses_to_overwrite_unexpected_shapes() {
        assert!(with_server(json!(["liste"]), Path::new(EXE)).is_err());
        assert!(with_server(json!({ "mcpServers": "texte" }), Path::new(EXE)).is_err());
    }

    #[test]
    fn installing_twice_gives_the_same_result() {
        let once = with_server(json!({}), Path::new(EXE)).unwrap();
        let twice = with_server(once.clone(), Path::new(EXE)).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn removal_only_touches_our_entry() {
        let installed = with_server(json!({ "mcpServers": { "filesystem": { "command": "npx" } } }), Path::new(EXE)).unwrap();
        let removed = without_server(installed).unwrap();
        assert!(removed["mcpServers"].get(SERVER_NAME).is_none());
        assert_eq!(removed["mcpServers"]["filesystem"]["command"], "npx");
    }

    #[test]
    fn removal_drops_an_emptied_mcp_servers_block() {
        let installed = with_server(json!({ "preferences": {} }), Path::new(EXE)).unwrap();
        let removed = without_server(installed).unwrap();
        assert!(removed.get("mcpServers").is_none());
        assert!(removed.get("preferences").is_some());
    }

    #[test]
    fn inspection_detects_stale_paths() {
        let installed = with_server(json!({}), Path::new(EXE)).unwrap();
        assert_eq!(inspect(&installed, Path::new(EXE)), (true, true));
        assert_eq!(inspect(&installed, Path::new(r"D:\Autre\katadacho.exe")), (true, false));
        assert_eq!(inspect(&json!({}), Path::new(EXE)), (false, false));
    }

    #[test]
    fn writes_a_backup_once_and_roundtrips() {
        let dir = std::env::temp_dir().join(format!("katadacho-mcp-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("claude_desktop_config.json");
        fs::write(&path, r#"{"preferences":{"a":1}}"#).unwrap();

        let updated = with_server(read_config(&path).unwrap(), Path::new(EXE)).unwrap();
        write_config(&path, &updated).unwrap();
        // Une seconde écriture ne doit pas remplacer la sauvegarde d'origine.
        write_config(&path, &updated).unwrap();

        let backup = fs::read_to_string(dir.join("claude_desktop_config.json.katadacho-backup")).unwrap();
        assert_eq!(backup, r#"{"preferences":{"a":1}}"#);
        assert_eq!(read_config(&path).unwrap()["mcpServers"][SERVER_NAME]["command"], EXE);
        fs::remove_dir_all(&dir).unwrap();
    }
}
