//! Cœur du protocole MCP (Model Context Protocol) : JSON-RPC 2.0 sur des messages d'une ligne.
//! Aucune entrée-sortie ici : la fonction `handle` reçoit un message et rend la réponse, ce qui
//! permet de la tester sans lancer de processus.

use serde_json::{json, Value};

/// Versions du protocole que ce serveur sait parler, de la plus récente à la plus ancienne.
const SUPPORTED_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

pub const TOOL_NOTIFY: &str = "notify_island";
pub const TOOL_ASK: &str = "ask_user_in_island";

const ERROR_METHOD_NOT_FOUND: i64 = -32601;
const ERROR_INVALID_PARAMS: i64 = -32602;

/// Ce que le serveur peut demander à l'application principale.
pub trait Bridge {
    fn notify(&self, title: &str, summary: &str, status: &str) -> Result<(), String>;
    fn ask(&self, question: &str, options: &[String]) -> Result<String, String>;
}

fn tools() -> Value {
    json!([
        {
            "name": TOOL_NOTIFY,
            "description": "Shows a notification in the island (the pill at the top of the user's screen). Use it when a long task finishes, or when something deserves to be seen without the user looking at this window.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Short title, one line." },
                    "summary": { "type": "string", "description": "Summary in one or two sentences." },
                    "status": {
                        "type": "string",
                        "enum": ["done", "waiting", "error", "info"],
                        "description": "done: finished, waiting: waiting for the user, error: failed, info: plain information."
                    }
                },
                "required": ["title", "summary"]
            }
        },
        {
            "name": TOOL_ASK,
            "description": "Asks the user a question in the island, with answer buttons, and waits for their choice. Use it for a quick decision (yes or no, picking among a few options) without making them come back to this window.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "question": { "type": "string", "description": "The question, short and clear." },
                    "options": {
                        "type": "array",
                        "items": { "type": "string" },
                        "minItems": 2,
                        "maxItems": 4,
                        "description": "The suggested answers, 2 to 4 buttons."
                    }
                },
                "required": ["question", "options"]
            }
        }
    ])
}

fn success(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn failure(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn tool_text(text: &str, is_error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": text }], "isError": is_error })
}

fn string_arg<'a>(arguments: &'a Value, key: &str) -> Option<&'a str> {
    arguments.get(key).and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty())
}

fn call_tool(name: &str, arguments: &Value, bridge: &dyn Bridge) -> Value {
    match name {
        TOOL_NOTIFY => {
            let (Some(title), Some(summary)) = (string_arg(arguments, "title"), string_arg(arguments, "summary")) else {
                return tool_text("The title and summary fields are required.", true);
            };
            let status = string_arg(arguments, "status").unwrap_or("info");
            match bridge.notify(title, summary, status) {
                Ok(()) => tool_text("Notification shown in the island.", false),
                Err(error) => tool_text(&format!("Impossible d'afficher la notification : {error}"), true),
            }
        }
        TOOL_ASK => {
            let Some(question) = string_arg(arguments, "question") else {
                return tool_text("The question field is required.", true);
            };
            let options: Vec<String> = arguments
                .get("options")
                .and_then(Value::as_array)
                .map(|items| {
                    items.iter().filter_map(Value::as_str).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
                })
                .unwrap_or_default();
            if !(2..=4).contains(&options.len()) {
                return tool_text("Il faut entre 2 et 4 options non vides.", true);
            }
            match bridge.ask(question, &options) {
                Ok(answer) => tool_text(&answer, false),
                Err(error) => tool_text(&format!("No answer: {error}"), true),
            }
        }
        other => tool_text(&format!("Outil inconnu : {other}"), true),
    }
}

/// Traite un message. Retourne `None` pour une notification JSON-RPC (sans `id`), qui n'attend pas de réponse.
pub fn handle(message: &Value, bridge: &dyn Bridge) -> Option<Value> {
    let method = message.get("method").and_then(Value::as_str)?;
    let id = message.get("id")?;

    Some(match method {
        "initialize" => {
            let requested = message.pointer("/params/protocolVersion").and_then(Value::as_str).unwrap_or("");
            let version = SUPPORTED_VERSIONS.iter().find(|v| **v == requested).unwrap_or(&SUPPORTED_VERSIONS[0]);
            success(
                id,
                json!({
                    "protocolVersion": version,
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "katadacho-island", "version": env!("CARGO_PKG_VERSION") }
                }),
            )
        }
        "ping" => success(id, json!({})),
        "tools/list" => success(id, json!({ "tools": tools() })),
        "tools/call" => {
            let Some(name) = message.pointer("/params/name").and_then(Value::as_str) else {
                return Some(failure(id, ERROR_INVALID_PARAMS, "Nom d'outil manquant"));
            };
            let arguments = message.pointer("/params/arguments").cloned().unwrap_or_else(|| json!({}));
            success(id, call_tool(name, &arguments, bridge))
        }
        other => failure(id, ERROR_METHOD_NOT_FOUND, &format!("Méthode inconnue : {other}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct FakeBridge {
        notified: RefCell<Vec<(String, String, String)>>,
        answer: Option<String>,
    }

    impl Bridge for FakeBridge {
        fn notify(&self, title: &str, summary: &str, status: &str) -> Result<(), String> {
            self.notified.borrow_mut().push((title.into(), summary.into(), status.into()));
            Ok(())
        }
        fn ask(&self, _question: &str, options: &[String]) -> Result<String, String> {
            self.answer.clone().or_else(|| options.first().cloned()).ok_or_else(|| "aucune réponse".to_string())
        }
    }

    fn request(method: &str, params: Value) -> Value {
        json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params })
    }

    #[test]
    fn initialize_echoes_a_supported_version() {
        let reply = handle(&request("initialize", json!({ "protocolVersion": "2025-03-26" })), &FakeBridge::default()).unwrap();
        assert_eq!(reply["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(reply["result"]["serverInfo"]["name"], "katadacho-island");
        assert!(reply["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn initialize_falls_back_to_the_latest_version() {
        let reply = handle(&request("initialize", json!({ "protocolVersion": "1999-01-01" })), &FakeBridge::default()).unwrap();
        assert_eq!(reply["result"]["protocolVersion"], SUPPORTED_VERSIONS[0]);
    }

    #[test]
    fn notifications_get_no_reply() {
        let message = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert!(handle(&message, &FakeBridge::default()).is_none());
    }

    #[test]
    fn lists_both_tools_with_schemas() {
        let reply = handle(&request("tools/list", json!({})), &FakeBridge::default()).unwrap();
        let names: Vec<&str> = reply["result"]["tools"].as_array().unwrap().iter().filter_map(|t| t["name"].as_str()).collect();
        assert_eq!(names, vec![TOOL_NOTIFY, TOOL_ASK]);
        assert!(reply["result"]["tools"][0]["inputSchema"]["required"].is_array());
    }

    #[test]
    fn notify_reaches_the_bridge_with_default_status() {
        let bridge = FakeBridge::default();
        let params = json!({ "name": TOOL_NOTIFY, "arguments": { "title": "Build terminé", "summary": "0 erreur" } });
        let reply = handle(&request("tools/call", params), &bridge).unwrap();
        assert_eq!(reply["result"]["isError"], false);
        assert_eq!(bridge.notified.borrow()[0], ("Build terminé".into(), "0 erreur".into(), "info".into()));
    }

    #[test]
    fn notify_requires_title_and_summary() {
        let params = json!({ "name": TOOL_NOTIFY, "arguments": { "title": "  " } });
        let reply = handle(&request("tools/call", params), &FakeBridge::default()).unwrap();
        assert_eq!(reply["result"]["isError"], true);
    }

    #[test]
    fn ask_returns_the_chosen_option() {
        let bridge = FakeBridge { answer: Some("Non".into()), ..Default::default() };
        let params = json!({ "name": TOOL_ASK, "arguments": { "question": "Continuer ?", "options": ["Oui", "Non"] } });
        let reply = handle(&request("tools/call", params), &bridge).unwrap();
        assert_eq!(reply["result"]["content"][0]["text"], "Non");
        assert_eq!(reply["result"]["isError"], false);
    }

    #[test]
    fn ask_rejects_a_wrong_number_of_options() {
        let params = json!({ "name": TOOL_ASK, "arguments": { "question": "?", "options": ["Seul"] } });
        let reply = handle(&request("tools/call", params), &FakeBridge::default()).unwrap();
        assert_eq!(reply["result"]["isError"], true);
    }

    #[test]
    fn unknown_methods_and_tools_are_reported() {
        let reply = handle(&request("resources/list", json!({})), &FakeBridge::default()).unwrap();
        assert_eq!(reply["error"]["code"], ERROR_METHOD_NOT_FOUND);
        let params = json!({ "name": "inconnu", "arguments": {} });
        let reply = handle(&request("tools/call", params), &FakeBridge::default()).unwrap();
        assert_eq!(reply["result"]["isError"], true);
    }
}
