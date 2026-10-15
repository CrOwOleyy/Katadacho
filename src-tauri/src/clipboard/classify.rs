/// Type de contenu stocké dans la colonne `content_type`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContentKind {
    Text,
    Url,
    Color,
    Code,
    Image,
}

impl ContentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ContentKind::Text => "text",
            ContentKind::Url => "url",
            ContentKind::Color => "color",
            ContentKind::Code => "code",
            ContentKind::Image => "image",
        }
    }
}

/// Indices de code source : plusieurs doivent apparaître pour classer en `Code`.
const CODE_MARKERS: &[&str] = &[
    "{", "}", ";", "=>", "()", "fn ", "function ", "def ", "class ", "import ", "const ", "let ",
    "return ", "</", "/>", "#include", "public ", "self.", "std::",
];

pub fn classify_text(text: &str) -> ContentKind {
    let trimmed = text.trim();

    if is_hex_color(trimmed) {
        return ContentKind::Color;
    }
    if is_url(trimmed) {
        return ContentKind::Url;
    }

    let markers = CODE_MARKERS.iter().filter(|marker| trimmed.contains(**marker)).count();
    let multiline = trimmed.contains('\n');
    if markers >= 3 || (multiline && markers >= 2) {
        return ContentKind::Code;
    }

    ContentKind::Text
}

fn is_hex_color(text: &str) -> bool {
    let Some(digits) = text.strip_prefix('#') else {
        return false;
    };
    matches!(digits.len(), 3 | 6 | 8) && digits.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_url(text: &str) -> bool {
    if text.chars().any(char::is_whitespace) {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("www.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_hex_colors() {
        assert_eq!(classify_text("#ff8800"), ContentKind::Color);
        assert_eq!(classify_text("  #FFF  "), ContentKind::Color);
        assert_eq!(classify_text("#12345"), ContentKind::Text);
        assert_eq!(classify_text("#gggggg"), ContentKind::Text);
    }

    #[test]
    fn detects_urls() {
        assert_eq!(classify_text("https://example.com/page?q=1"), ContentKind::Url);
        assert_eq!(classify_text("www.example.com"), ContentKind::Url);
        assert_eq!(classify_text("voir https://example.com"), ContentKind::Text);
    }

    #[test]
    fn detects_code() {
        let rust = "fn main() {\n    let x = 1;\n    println!(\"{}\", x);\n}";
        assert_eq!(classify_text(rust), ContentKind::Code);
        assert_eq!(classify_text("Bonjour, comment ça va ?"), ContentKind::Text);
    }
}
