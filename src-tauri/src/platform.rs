//! Outils communs pour les fonctionnalités qui n'existent pas encore sur tous les systèmes.
// Sous Windows tout est implémenté : ces outils ne servent qu'aux autres systèmes.
#![cfg_attr(windows, allow(dead_code))]

/// Nom lisible du système courant.
pub fn os_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "Windows",
        "linux" => "Linux",
        "macos" => "macOS",
        other => other,
    }
}

/// Message d'erreur explicite pour une fonctionnalité absente du système courant.
/// Les commandes concernées le renvoient plutôt que d'inventer des données.
pub fn unsupported(feature: &str) -> String {
    format!("{feature} n'est pas encore disponible sur {}", os_name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_names_the_feature_and_system() {
        let message = unsupported("Le contrôle du volume");
        assert!(message.starts_with("Le contrôle du volume n'est pas encore disponible sur "));
        assert!(message.ends_with(os_name()));
    }
}
