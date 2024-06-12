use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "camelCase")
)]
pub struct DocumentCreateResponse {
    pub uuid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub vault_uuid: String,
}
