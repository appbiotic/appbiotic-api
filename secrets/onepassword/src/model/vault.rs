use chrono::{DateTime, Utc};

use crate::VaultType;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub struct Vault {
    pub id: String,
    pub name: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<String>,

    pub attribute_version: u64,
    pub content_version: u64,
    pub items: u64,

    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub vault_type: VaultType,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
