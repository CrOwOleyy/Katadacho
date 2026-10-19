//! Types partagés par toutes les implémentations de lecture des notifications.

use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Toast {
    pub id: u32,
    pub app_name: String,
    pub title: String,
    pub body: String,
    /// Horodatage Unix en millisecondes.
    pub created_ms: i64,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Access {
    Allowed,
    Denied,
    /// L'utilisateur n'a pas encore répondu à la demande d'autorisation.
    Unspecified,
}
