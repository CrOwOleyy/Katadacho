//! Temps écoulé depuis la dernière action de l'utilisateur (clavier ou souris), sur tout le système.
//! Sert à endormir la mascotte quand personne n'utilise l'ordinateur.

/// Secondes depuis la dernière entrée clavier ou souris, toutes applications confondues.
#[cfg(windows)]
fn idle_seconds() -> Result<u64, String> {
    use std::mem::size_of;

    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    let mut info = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    if !unsafe { GetLastInputInfo(&mut info) }.as_bool() {
        return Err("Lecture de l'inactivité impossible".to_string());
    }
    // Les deux compteurs bouclent tous les 49 jours : la soustraction modulaire reste juste.
    let elapsed_ms = unsafe { GetTickCount() }.wrapping_sub(info.dwTime);
    Ok(u64::from(elapsed_ms) / 1000)
}

#[cfg(not(windows))]
fn idle_seconds() -> Result<u64, String> {
    Err(format!("La mesure de l'inactivité n'est pas encore disponible sur {}", std::env::consts::OS))
}

#[tauri::command]
pub fn get_idle_seconds() -> Result<u64, String> {
    idle_seconds()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn reads_a_plausible_idle_time() {
        let seconds = idle_seconds().expect("inactivité lisible");
        // Moins d'un jour : la session de test est forcément active.
        assert!(seconds < 86_400);
    }
}
