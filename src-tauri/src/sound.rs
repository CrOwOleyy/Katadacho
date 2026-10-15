//! Sons système Windows, partagés par le minuteur et les notifications.

use windows::core::w;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC};

/// Joue le son de notification de Windows, sans bloquer.
pub fn play_notification() {
    unsafe {
        let _ = PlaySoundW(w!("SystemNotification"), None, SND_ALIAS | SND_ASYNC);
    }
}
