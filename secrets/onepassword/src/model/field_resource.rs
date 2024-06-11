use std::{fmt, str::FromStr, sync::OnceLock};

use regex::Regex;

use crate::OnePasswordError;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub struct FieldResource {
    pub vault: String,
    pub item: String,
    pub field: String,
}

fn re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^op://([[:alnum:]]+)/([[:alnum:]]+)/([[:alnum:]]+)$").unwrap())
}

impl FromStr for FieldResource {
    type Err = OnePasswordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = re().captures(s) {
            Ok(Self {
                vault: captures.get(1).unwrap().as_str().to_owned(),
                item: captures.get(2).unwrap().as_str().to_owned(),
                field: captures.get(3).unwrap().as_str().to_owned(),
            })
        } else {
            Err(OnePasswordError::ResourceParsingFailed)
        }
    }
}

impl fmt::Display for FieldResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "op://{}/{}/{}",
            self.vault, self.item, self.field
        ))
    }
}
