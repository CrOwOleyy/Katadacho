//! Canal local entre le processus serveur MCP (lancé par Claude Desktop) et l'application déjà ouverte.
//! L'application écoute sur `127.0.0.1` avec un port aléatoire et un jeton secret, écrits dans
//! `bridge.json` du dossier de données. Le serveur MCP lit ce fichier, se connecte et envoie une
//! requête d'une ligne JSON. Seuls les processus de l'utilisateur peuvent lire le jeton.

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};

use super::protocol::Bridge;

pub const EVENT_AI_NOTIFY: &str = "ai-notify";
pub const EVENT_AI_QUESTION: &str = "ai-question";
pub const EVENT_AI_QUESTION_CLOSED: &str = "ai-question-closed";
const BRIDGE_FILE: &str = "bridge.json";
/// Temps laissé à l'utilisateur pour répondre à une question.
const ANSWER_TIMEOUT: Duration = Duration::from_secs(120);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
/// Plafond de taille d'une requête, pour qu'un client malveillant ne remplisse pas la mémoire.
const MAX_REQUEST_BYTES: u64 = 64 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct BridgeInfo {
    port: u16,
    token: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiNotification {
    pub title: String,
    pub summary: String,
    pub status: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AiQuestion {
    pub id: u64,
    pub question: String,
    pub options: Vec<String>,
}

/// Questions en attente : l'identifiant renvoie vers le canal qui réveille la requête bloquée.
#[derive(Default)]
pub struct PendingQuestions {
    next_id: AtomicU64,
    waiting: Mutex<HashMap<u64, Sender<String>>>,
}

impl PendingQuestions {
    fn register(&self) -> (u64, mpsc::Receiver<String>) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let (sender, receiver) = mpsc::channel();
        self.waiting.lock().unwrap_or_else(|e| e.into_inner()).insert(id, sender);
        (id, receiver)
    }

    fn forget(&self, id: u64) {
        self.waiting.lock().unwrap_or_else(|e| e.into_inner()).remove(&id);
    }

    /// Transmet la réponse. Retourne `false` si la question n'existe plus (expirée ou déjà répondue).
    pub fn answer(&self, id: u64, answer: String) -> bool {
        let sender = self.waiting.lock().unwrap_or_else(|e| e.into_inner()).remove(&id);
        sender.is_some_and(|sender| sender.send(answer).is_ok())
    }
}

fn bridge_path(data_dir: &Path) -> PathBuf {
    data_dir.join(BRIDGE_FILE)
}

/// Jeton aléatoire de 128 bits, tiré du générateur cryptographique du système.
fn random_token() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("générateur aléatoire du système indisponible");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn read_line_limited(stream: &TcpStream) -> Option<String> {
    let mut line = String::new();
    BufReader::new(stream.take(MAX_REQUEST_BYTES)).read_line(&mut line).ok()?;
    (!line.trim().is_empty()).then_some(line)
}

use std::io::Read;

fn reply(mut stream: &TcpStream, value: Value) {
    let _ = writeln!(stream, "{value}");
}

fn handle_connection(app: &AppHandle, token: &str, stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(CONNECT_TIMEOUT));
    let Some(line) = read_line_limited(&stream) else {
        return;
    };
    let Ok(request) = serde_json::from_str::<Value>(&line) else {
        return reply(&stream, json!({ "ok": false, "error": "JSON invalide" }));
    };
    if request.get("token").and_then(Value::as_str) != Some(token) {
        return reply(&stream, json!({ "ok": false, "error": "Jeton refusé" }));
    }
    // Une question peut attendre la réponse de l'utilisateur : plus de délai de lecture.
    let _ = stream.set_read_timeout(None);

    let text = |key: &str| request.get(key).and_then(Value::as_str).unwrap_or_default().to_string();
    match request.get("kind").and_then(Value::as_str) {
        Some("notify") => {
            let notification = AiNotification { title: text("title"), summary: text("summary"), status: text("status") };
            let _ = app.emit(EVENT_AI_NOTIFY, notification);
            reply(&stream, json!({ "ok": true }));
        }
        Some("ask") => {
            let options: Vec<String> = request
                .get("options")
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(Value::as_str).map(String::from).collect())
                .unwrap_or_default();
            let pending = app.state::<PendingQuestions>();
            let (id, receiver) = pending.register();
            let _ = app.emit(EVENT_AI_QUESTION, AiQuestion { id, question: text("question"), options });

            match receiver.recv_timeout(ANSWER_TIMEOUT) {
                Ok(answer) => reply(&stream, json!({ "ok": true, "answer": answer })),
                Err(_) => {
                    pending.forget(id);
                    let _ = app.emit(EVENT_AI_QUESTION_CLOSED, id);
                    reply(&stream, json!({ "ok": false, "error": "L'utilisateur n'a pas répondu à temps" }));
                }
            }
        }
        _ => reply(&stream, json!({ "ok": false, "error": "Requête inconnue" })),
    }
}

/// Démarre l'écoute locale. En cas d'échec le reste de l'application continue : seul le pont est absent.
pub fn start(app: AppHandle, data_dir: PathBuf) {
    let listener = match TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)) {
        Ok(listener) => listener,
        Err(error) => return eprintln!("MCP bridge unavailable: {error}"),
    };
    let Ok(address) = listener.local_addr() else {
        return;
    };
    let token = random_token();
    let info = BridgeInfo { port: address.port(), token: token.clone() };

    if let Err(error) = fs::create_dir_all(&data_dir)
        .and_then(|_| fs::write(bridge_path(&data_dir), serde_json::to_string(&info).unwrap_or_default()))
    {
        return eprintln!("MCP bridge file not written: {error}");
    }

    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let app = app.clone();
            let token = token.clone();
            // Un thread par connexion : une question en attente ne bloque pas les notifications.
            thread::spawn(move || handle_connection(&app, &token, stream));
        }
    });
}

/// Côté serveur MCP : parle à l'application ouverte, ou explique qu'elle n'est pas lancée.
pub struct AppBridge {
    data_dir: PathBuf,
}

impl AppBridge {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn send(&self, mut request: Value, wait: Duration) -> Result<Value, String> {
        let text = fs::read_to_string(bridge_path(&self.data_dir))
            .map_err(|_| "Katadacho n'est pas lancé".to_string())?;
        let info: BridgeInfo = serde_json::from_str(&text).map_err(|_| "Fichier du pont illisible".to_string())?;
        request["token"] = json!(info.token);

        let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, info.port);
        let mut stream = TcpStream::connect_timeout(&address.into(), CONNECT_TIMEOUT)
            .map_err(|_| "Katadacho n'est pas lancé".to_string())?;
        stream.set_read_timeout(Some(wait)).map_err(|e| e.to_string())?;
        writeln!(stream, "{request}").map_err(|e| e.to_string())?;

        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).map_err(|e| e.to_string())?;
        let response: Value = serde_json::from_str(&line).map_err(|_| "Réponse illisible".to_string())?;
        if response.get("ok") == Some(&json!(true)) {
            Ok(response)
        } else {
            Err(response.get("error").and_then(Value::as_str).unwrap_or("Erreur inconnue").to_string())
        }
    }
}

impl Bridge for AppBridge {
    fn notify(&self, title: &str, summary: &str, status: &str) -> Result<(), String> {
        let request = json!({ "kind": "notify", "title": title, "summary": summary, "status": status });
        self.send(request, CONNECT_TIMEOUT).map(|_| ())
    }

    fn ask(&self, question: &str, options: &[String]) -> Result<String, String> {
        let request = json!({ "kind": "ask", "question": question, "options": options });
        let response = self.send(request, ANSWER_TIMEOUT + CONNECT_TIMEOUT)?;
        response.get("answer").and_then(Value::as_str).map(String::from).ok_or_else(|| "Réponse vide".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_long_and_differ_between_calls() {
        let (a, b) = (random_token(), random_token());
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
    }

    #[test]
    fn answers_reach_the_waiting_request_once() {
        let pending = PendingQuestions::default();
        let (id, receiver) = pending.register();
        assert!(pending.answer(id, "Oui".into()));
        assert_eq!(receiver.recv_timeout(Duration::from_millis(100)).unwrap(), "Oui");
        // Une seconde réponse à la même question est refusée.
        assert!(!pending.answer(id, "Non".into()));
    }

    #[test]
    fn answering_an_unknown_question_is_refused() {
        assert!(!PendingQuestions::default().answer(42, "Oui".into()));
    }

    #[test]
    fn without_a_running_app_the_bridge_says_so() {
        let dir = std::env::temp_dir().join(format!("katadacho-bridge-test-{}", std::process::id()));
        let bridge = AppBridge::new(dir);
        assert_eq!(bridge.notify("t", "s", "info"), Err("Katadacho n'est pas lancé".to_string()));
    }
}
