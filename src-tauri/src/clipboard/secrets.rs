//! Repère les textes copiés qui ressemblent à un secret : clé d'API, jeton, mot de passe.
//! Ces éléments sont masqués dans l'historique puis effacés automatiquement.

/// Préfixes de clés et de jetons très répandus.
const KNOWN_PREFIXES: &[&str] = &[
    "sk-", "sk_live_", "sk_test_", "pk_live_", "rk_live_", "ghp_", "gho_", "ghs_", "ghu_", "github_pat_",
    "glpat-", "xoxb-", "xoxp-", "AKIA", "ASIA", "AIza", "ya29.", "hf_", "npm_",
];

const MIN_PASSWORD_LEN: usize = 10;
const MAX_PASSWORD_LEN: usize = 64;
const MIN_TOKEN_LEN: usize = 16;
const MAX_TOKEN_LEN: usize = 4096;

fn is_jwt(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    text.starts_with("eyJ")
        && parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
}

fn is_email(text: &str) -> bool {
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

/// Un mot de passe typique : une seule « chaîne », 10 à 64 caractères, au moins trois familles
/// parmi minuscules, majuscules, chiffres et symboles.
fn looks_like_password(text: &str) -> bool {
    let length = text.chars().count();
    if !(MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&length) {
        return false;
    }
    // Chemins, liens et adresses ont des symboles mais ne sont pas des secrets.
    if text.contains('\\') || text.contains("://") || text.starts_with("www.") || is_email(text) {
        return false;
    }

    let lower = text.chars().any(|c| c.is_lowercase());
    let upper = text.chars().any(|c| c.is_uppercase());
    let digit = text.chars().any(|c| c.is_ascii_digit());
    let symbol = text.chars().any(|c| !c.is_alphanumeric());
    [lower, upper, digit, symbol].iter().filter(|present| **present).count() >= 3
}

pub fn looks_secret(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) {
        return false;
    }

    let length = trimmed.chars().count();
    if (MIN_TOKEN_LEN..=MAX_TOKEN_LEN).contains(&length)
        && (KNOWN_PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix)) || is_jwt(trimmed))
    {
        return true;
    }

    looks_like_password(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_api_keys_and_tokens() {
        assert!(looks_secret("sk-proj-4f8Hd92kLmQ0rTz7XbWcYvA1"));
        assert!(looks_secret("ghp_1234567890abcdefghijABCDEFGHIJ"));
        assert!(looks_secret("AKIAIOSFODNN7EXAMPLE"));
        assert!(looks_secret("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjMifQ.dGVzdHNpZ25hdHVyZQ"));
    }

    #[test]
    fn detects_password_like_strings() {
        assert!(looks_secret("Tr0ub4dor&3xyz"));
        assert!(looks_secret("p@ssW0rd2026!"));
    }

    #[test]
    fn ignores_ordinary_text() {
        assert!(!looks_secret("Bonjour, comment ça va ?"));
        assert!(!looks_secret("https://example.com/Page?Id=42"));
        assert!(!looks_secret("C:\\Users\\Ryad\\Documents"));
        assert!(!looks_secret("ryad.younes@epitech.eu"));
        assert!(!looks_secret("useEffect"));
        assert!(!looks_secret("#FF8800"));
        assert!(!looks_secret("0612345678"));
        assert!(!looks_secret("short1!A"));
    }
}
