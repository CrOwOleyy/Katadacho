//! Outils de pochette indépendants du système : type d'image et chemin d'une URL `file://`.
// Sous Windows la pochette vient du système : ces fonctions ne servent qu'aux autres systèmes.
#![cfg_attr(windows, allow(dead_code))]

/// Type MIME d'une image reconnue à ses premiers octets.
pub fn sniff_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF8") {
        Some("image/gif")
    } else if bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Chemin local d'une URL `file:///chemin/avec%20espaces.jpg`.
pub fn file_path_from_url(url: &str) -> Option<String> {
    let rest = url.strip_prefix("file://")?;
    // `file://localhost/chemin` est permis par la norme.
    let rest = rest.strip_prefix("localhost").unwrap_or(rest);
    if !rest.starts_with('/') {
        return None;
    }

    let bytes = rest.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (hex(bytes[index + 1]), hex(bytes[index + 2])) {
                decoded.push(high * 16 + low);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_is_detected_from_magic_bytes() {
        assert_eq!(sniff_mime(&[0x89, b'P', b'N', b'G', 0, 0]), Some("image/png"));
        assert_eq!(sniff_mime(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(sniff_mime(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
        assert_eq!(sniff_mime(b"<html>"), None);
    }

    #[test]
    fn file_urls_are_decoded() {
        assert_eq!(file_path_from_url("file:///home/moi/Ma%20Musique/a%C3%A9.jpg").as_deref(), Some("/home/moi/Ma Musique/aé.jpg"));
        assert_eq!(file_path_from_url("file://localhost/tmp/x.png").as_deref(), Some("/tmp/x.png"));
        assert_eq!(file_path_from_url("https://exemple.org/x.png"), None);
        assert_eq!(file_path_from_url("file:///tmp/50%"), Some("/tmp/50%".to_string()));
    }
}
