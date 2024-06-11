#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "SCREAMING_SNAKE_CASE")
)]
pub enum PasswordStrength {
    Terrible,
    #[cfg_attr(feature = "serde", serde(untagged))]
    Unrecognized(String),
}
