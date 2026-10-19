//! Son de notification du système, partagé par le minuteur et les notifications.

/// Joue le son de notification du système, sans bloquer.
#[cfg(windows)]
pub fn play_notification() {
    use windows::core::w;
    use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC};

    unsafe {
        let _ = PlaySoundW(w!("SystemNotification"), None, SND_ALIAS | SND_ASYNC);
    }
}

/// Joue le son de notification du thème freedesktop, sans bloquer. Essaie les lecteurs présents
/// sur la plupart des distributions ; si aucun n'est installé, il n'y a simplement pas de son.
#[cfg(target_os = "linux")]
pub fn play_notification() {
    use std::process::{Command, Stdio};

    const FREEDESKTOP_SOUND: &str = "/usr/share/sounds/freedesktop/stereo/complete.oga";

    let candidates: [(&str, &[&str]); 3] = [
        ("canberra-gtk-play", &["--id", "complete"]),
        ("paplay", &[FREEDESKTOP_SOUND]),
        ("pw-play", &[FREEDESKTOP_SOUND]),
    ];

    std::thread::spawn(move || {
        for (program, arguments) in candidates {
            let played = Command::new(program)
                .args(arguments)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success());
            if played {
                return;
            }
        }
    });
}

/// Joue le son « Glass » de macOS, sans bloquer.
#[cfg(target_os = "macos")]
pub fn play_notification() {
    let _ = std::process::Command::new("afplay").arg("/System/Library/Sounds/Glass.aiff").spawn();
}

/// Autres systèmes : pas de son système connu.
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn play_notification() {}
