#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "SCREAMING_SNAKE_CASE")
)]
pub enum CharacterSet {
    Letters,
    Digits,
    Symbols,
    #[cfg_attr(feature = "serde", serde(untagged))]
    Unrecognized(String),
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn character_set_deserializes() {
        let value: CharacterSet = serde_json::from_value(json!("LETTERS")).unwrap();
        assert_eq!(value, CharacterSet::Letters);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn character_set_deserializes_unknown() {
        let value: CharacterSet = serde_json::from_value(json!("EMOJI")).unwrap();
        assert_eq!(value, CharacterSet::Unrecognized("EMOJI".to_owned()));
    }
}
