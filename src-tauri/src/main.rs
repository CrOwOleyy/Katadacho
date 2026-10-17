// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Relancé élevé pour le nettoyage profond de la mémoire : on ne démarre pas l'application.
    #[cfg(windows)]
    if std::env::args().any(|argument| argument == katadacho_lib::memory::deep::CLI_FLAG) {
        std::process::exit(katadacho_lib::memory::deep::run_cli());
    }
    katadacho_lib::run()
}
