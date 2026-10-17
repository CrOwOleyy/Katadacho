//! Boucle du serveur MCP en mode `katadacho.exe --mcp-server`, lancé par Claude Desktop.
//! Le transport « stdio » de MCP échange un message JSON par ligne sur l'entrée et la sortie standard.
//! La sortie standard est réservée au protocole : tout message de diagnostic va sur la sortie d'erreur.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use serde_json::{json, Value};

use super::bridge::AppBridge;
use super::protocol::{handle, Bridge};

const ERROR_PARSE: i64 = -32700;

/// Lit les messages un par un et écrit les réponses. Retourne quand l'entrée est fermée.
pub fn serve(input: impl BufRead, mut output: impl Write, bridge: &dyn Bridge) {
    for line in input.lines() {
        let Ok(line) = line else {
            break;
        };
        // Certains clients (PowerShell, éditeurs) ajoutent un BOM UTF-8 au début du flux.
        let line = line.trim_start_matches('\u{feff}').to_string();
        if line.trim().is_empty() {
            continue;
        }

        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(message) => handle(&message, bridge),
            Err(_) => Some(json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": { "code": ERROR_PARSE, "message": "JSON invalide" }
            })),
        };

        if let Some(reply) = reply {
            if writeln!(output, "{reply}").and_then(|_| output.flush()).is_err() {
                break;
            }
        }
    }
}

/// Point d'entrée du processus. Retourne le code de sortie.
pub fn run(data_dir: PathBuf) -> i32 {
    let bridge = AppBridge::new(data_dir);
    serve(std::io::stdin().lock(), std::io::stdout().lock(), &bridge);
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct Echo;
    impl Bridge for Echo {
        fn notify(&self, _: &str, _: &str, _: &str) -> Result<(), String> {
            Ok(())
        }
        fn ask(&self, _: &str, options: &[String]) -> Result<String, String> {
            Ok(options[0].clone())
        }
    }

    fn run_lines(input: &str) -> Vec<Value> {
        let mut output = Vec::new();
        serve(Cursor::new(input.to_string()), &mut output, &Echo);
        String::from_utf8(output).unwrap().lines().map(|line| serde_json::from_str(line).unwrap()).collect()
    }

    #[test]
    fn answers_a_full_session_one_line_per_message() {
        let replies = run_lines(concat!(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#, "\n",
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#, "\n",
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#, "\n",
        ));
        // La notification « initialized » n'attend pas de réponse : deux réponses pour trois messages.
        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0]["id"], 1);
        assert_eq!(replies[1]["result"]["tools"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn survives_garbage_and_blank_lines() {
        let replies = run_lines("\nceci n'est pas du json\n{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"ping\"}\n");
        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0]["error"]["code"], ERROR_PARSE);
        assert_eq!(replies[1]["id"], 7);
    }
}

#[cfg(test)]
mod bom_tests {
    use super::*;
    use std::io::Cursor;

    struct Silent;
    impl Bridge for Silent {
        fn notify(&self, _: &str, _: &str, _: &str) -> Result<(), String> {
            Ok(())
        }
        fn ask(&self, _: &str, options: &[String]) -> Result<String, String> {
            Ok(options[0].clone())
        }
    }

    #[test]
    fn accepts_a_utf8_bom_before_the_first_message() {
        let input = "\u{feff}{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n";
        let mut output = Vec::new();
        serve(Cursor::new(input.to_string()), &mut output, &Silent);
        let reply: Value = serde_json::from_str(String::from_utf8(output).unwrap().trim()).unwrap();
        assert_eq!(reply["id"], 1);
        assert!(reply.get("error").is_none());
    }
}
