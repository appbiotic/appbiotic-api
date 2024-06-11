#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "SCREAMING_SNAKE_CASE")
)]
pub enum VaultType {
    UserCreated,
    Personal,
    Everyone,
    Transfer,
    #[cfg_attr(feature = "serde", serde(untagged))]
    Unrecognized(String),
}
