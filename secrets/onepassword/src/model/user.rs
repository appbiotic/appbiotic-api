use chrono::{DateTime, Utc};

use crate::UserType;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub r#type: UserType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_auth_at: DateTime<Utc>,
}
