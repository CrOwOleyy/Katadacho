use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Matcher, Utf32Str};

use super::model::{EntryKind, LauncherEntry, LauncherResult};

/// Sous ce nombre de caractères, seules les applications sont proposées : une ou deux lettres
/// correspondraient à des milliers de fichiers sans intérêt.
const MIN_QUERY_FOR_FILES: usize = 2;
/// Les applications passent avant les fichiers à pertinence égale.
const APP_BONUS: u32 = 60;

fn is_application(kind: EntryKind) -> bool {
    matches!(kind, EntryKind::App | EntryKind::Uwp)
}

pub fn search(
    entries: &[LauncherEntry],
    matcher: &mut Matcher,
    query: &str,
    limit: usize,
) -> Vec<LauncherResult> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    let files_allowed = query.chars().count() >= MIN_QUERY_FOR_FILES;
    let mut buffer = Vec::new();

    let mut scored: Vec<(u32, &LauncherEntry)> = entries
        .iter()
        .filter_map(|entry| {
            if !files_allowed && !is_application(entry.kind) {
                return None;
            }
            let score = pattern.score(Utf32Str::new(&entry.name, &mut buffer), matcher)?;
            let bonus = if is_application(entry.kind) { APP_BONUS } else { 0 };
            Some((score + bonus, entry))
        })
        .collect();

    // À score égal, le nom le plus court est le plus proche de la requête.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.len().cmp(&b.1.name.len())));

    scored
        .into_iter()
        .take(limit)
        .map(|(score, entry)| LauncherResult {
            name: entry.name.clone(),
            path: entry.path.clone(),
            kind: entry.kind,
            score,
        })
        .collect()
}
