use std::{fmt, str::FromStr, sync::OnceLock};

use regex::Regex;

use crate::OnePasswordError;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub struct VaultResource {
    pub vault: String,
}

impl VaultResource {
    pub fn vault(&self) -> &str {
        &self.vault
    }
}

impl FromStr for VaultResource {
    type Err = OnePasswordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"^op://([[:alnum:]]+)$").unwrap());
        if let Some(captures) = re.captures(s) {
            Ok(Self {
                vault: captures.get(1).unwrap().as_str().to_owned(),
            })
        } else {
            Err(OnePasswordError::ResourceParsingFailed)
        }
    }
}

impl fmt::Display for VaultResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("op://{}", self.vault))
    }
}
