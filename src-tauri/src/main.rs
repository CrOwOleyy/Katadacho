// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let arguments: Vec<String> = std::env::args().collect();

    // Relancé élevé pour le nettoyage profond de la mémoire : on ne démarre pas l'application.
    if arguments.iter().any(|argument| argument == katadacho_lib::memory::deep::CLI_FLAG) {
        std::process::exit(katadacho_lib::memory::deep::run_cli());
    }
    // Lancé par Claude Desktop comme serveur MCP : on parle sur l'entrée et la sortie standard.
    if arguments.iter().any(|argument| argument == katadacho_lib::mcp::CLI_FLAG) {
        std::process::exit(katadacho_lib::mcp::run_cli());
    }
    katadacho_lib::run()
}
