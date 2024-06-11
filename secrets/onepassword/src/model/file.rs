use crate::FileSection;

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
pub struct File {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub content_path: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub section: Option<FileSection>,

    #[cfg(feature = "serde")]
    #[serde_as(as = "Option<Base64<Standard, Padded>>")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<u8>>,

    #[cfg(not(feature = "serde"))]
    pub content: Option<Vec<u8>>,
}

#[cfg(test)]
mod test {
    use base64::prelude::*;
    use serde_json::json;

    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn fileset_deserializes_with_content() {
        let content = "abc\n";
        let size: u64 = content.as_bytes().len().try_into().unwrap();
        let content_encoded = BASE64_STANDARD.encode(content);

        let value: File = serde_json::from_value(json!({
            "id": "abc-123",
            "name": "ABC.txt",
            "size": size,
            "content_path": "abc.txt",
            "content": content_encoded
        }))
        .unwrap();

        assert_eq!(value.content, Some(content.as_bytes().to_vec()));
    }
}
