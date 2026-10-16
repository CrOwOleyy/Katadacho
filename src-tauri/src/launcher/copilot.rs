//! Touche Copilot comme raccourci du lanceur. Elle envoie `Win + Maj + F23`, combinaison que
//! Windows réserve : un raccourci global classique est refusé. Un crochet clavier bas niveau voit
//! la touche avant Windows, l'intercepte, et masque le relâchement de `Win` pour que le menu
//! Démarrer ne s'ouvre pas.

use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_LWIN,
    VK_RWIN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HC_ACTION, KBDLLHOOKSTRUCT,
    LLKHF_INJECTED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

const VK_F23: u32 = 0x86;
/// Code de touche non attribué, utilisé pour « masquer » Win comme le fait AutoHotkey.
const VK_MASK: VIRTUAL_KEY = VIRTUAL_KEY(0xE8);

static ENABLED: AtomicBool = AtomicBool::new(false);
/// Empêche la répétition automatique de déclencher le lanceur plusieurs fois.
static HELD: AtomicBool = AtomicBool::new(false);
static PRESSES: OnceLock<Mutex<mpsc::Sender<()>>> = OnceLock::new();

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::SeqCst);
}

fn key(code: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: code,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { Default::default() },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn win_is_down() -> bool {
    let pressed = |vk: VIRTUAL_KEY| unsafe { GetAsyncKeyState(i32::from(vk.0)) } < 0;
    pressed(VK_LWIN) || pressed(VK_RWIN)
}

/// Une frappe neutre pendant que Win est enfoncé : Windows n'ouvre plus le menu Démarrer au relâchement.
fn mask_start_menu() {
    let inputs = [key(VK_MASK, false), key(VK_MASK, true)];
    unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
}

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && ENABLED.load(Ordering::SeqCst) {
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let injected = info.flags.0 & LLKHF_INJECTED.0 != 0;

        if info.vkCode == VK_F23 && !injected {
            let message = wparam.0 as u32;
            if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                if !HELD.swap(true, Ordering::SeqCst) {
                    if win_is_down() {
                        mask_start_menu();
                    }
                    if let Some(sender) = PRESSES.get() {
                        let _ = sender.lock().unwrap_or_else(|e| e.into_inner()).send(());
                    }
                }
            } else if message == WM_KEYUP || message == WM_SYSKEYUP {
                HELD.store(false, Ordering::SeqCst);
            }
            // La touche est consommée : Windows ne lance pas Copilot.
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Installe le crochet une seule fois. `on_press` est appelé hors du crochet, qui doit rester rapide.
pub fn start(on_press: impl Fn() + Send + 'static) {
    let (sender, receiver) = mpsc::channel::<()>();
    if PRESSES.set(Mutex::new(sender)).is_err() {
        return;
    }

    thread::spawn(move || {
        for () in receiver {
            on_press();
        }
    });

    thread::spawn(|| {
        let module = unsafe { GetModuleHandleW(None) }.ok();
        let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), module.map(Into::into), 0) };
        if let Err(error) = hook {
            eprintln!("copilot key hook unavailable: {error}");
            return;
        }

        // Un crochet bas niveau exige une boucle de messages sur le thread qui l'installe.
        let mut message = MSG::default();
        while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
            unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    });
}
