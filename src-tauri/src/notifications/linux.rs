//! Notifications sous Linux. Pas encore branché : aucune notification n'est relayée.

use tauri::AppHandle;

use super::Access;

pub fn start(_app: AppHandle) {}

pub fn access() -> Access {
    Access::Allowed
}
