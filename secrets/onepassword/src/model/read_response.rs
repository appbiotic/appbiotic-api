#[cfg(feature = "serde")]
use serde_with::{
    base64::{Base64, Standard},
    formats::Padded,
    serde_as,
};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    serde_as,
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub struct ReadResponse {
    #[cfg(feature = "serde")]
    #[serde_as(as = "Base64<Standard, Padded>")]
    pub content: Vec<u8>,

    #[cfg(not(feature = "serde"))]
    pub content: Vec<u8>,
}
