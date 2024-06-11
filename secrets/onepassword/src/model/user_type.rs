#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserType {
    Member,
    ServiceAccount,
    #[serde(untagged)]
    Unknown(String),
}
