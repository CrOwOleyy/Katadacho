//! Applications du Microsoft Store, listées via le dossier virtuel `AppsFolder` du Shell,
//! le même que celui qui alimente le menu Démarrer.

use windows::core::PWSTR;
use windows::Win32::System::Com::{CoTaskMemFree, IBindCtx};
use windows::Win32::UI::Shell::{
    IEnumShellItems, IShellItem, SHGetKnownFolderItem, BHID_EnumItems, FOLDERID_AppsFolder,
    KNOWN_FOLDER_FLAG, SIGDN_DESKTOPABSOLUTEPARSING, SIGDN_NORMALDISPLAY,
};

use super::model::{EntryKind, LauncherEntry};

/// Convertit une chaîne allouée par le Shell en `String` puis libère la mémoire.
fn take_string(text: PWSTR) -> Option<String> {
    let value = unsafe { text.to_string() }.ok();
    unsafe { CoTaskMemFree(Some(text.0 as *const std::ffi::c_void)) };
    value
}

/// Liste les applications UWP. Le COM doit déjà être initialisé sur le thread appelant.
pub fn scan() -> Vec<LauncherEntry> {
    let mut entries = Vec::new();

    let Ok(folder) = (unsafe { SHGetKnownFolderItem::<IShellItem>(&FOLDERID_AppsFolder, KNOWN_FOLDER_FLAG(0), None) })
    else {
        return entries;
    };
    let Ok(enumerator) = (unsafe { folder.BindToHandler::<_, IEnumShellItems>(None::<&IBindCtx>, &BHID_EnumItems) })
    else {
        return entries;
    };

    loop {
        let mut batch: [Option<IShellItem>; 1] = [None];
        let mut fetched = 0u32;
        if unsafe { enumerator.Next(&mut batch, Some(&mut fetched)) }.is_err() || fetched == 0 {
            break;
        }
        let Some(item) = batch[0].take() else {
            break;
        };

        let Some(parsing) = unsafe { item.GetDisplayName(SIGDN_DESKTOPABSOLUTEPARSING) }
            .ok()
            .and_then(take_string)
        else {
            continue;
        };
        // Un AppUserModelId d'application UWP contient « ! » (ex. Microsoft.WindowsCalculator_..!App).
        // Les applications de bureau y apparaissent aussi, mais déjà couvertes par le menu Démarrer.
        if !parsing.contains('!') {
            continue;
        }
        let Some(name) = unsafe { item.GetDisplayName(SIGDN_NORMALDISPLAY) }.ok().and_then(take_string) else {
            continue;
        };

        entries.push(LauncherEntry {
            name,
            path: format!("shell:AppsFolder\\{parsing}"),
            kind: EntryKind::Uwp,
        });
    }

    entries
}
