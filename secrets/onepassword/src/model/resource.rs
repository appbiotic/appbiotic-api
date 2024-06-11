#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "SCREAMING_SNAKE_CASE")
)]
pub enum Resource {
    Vault {
        vault_id: String,
    },
    Item {
        vault_id: String,
        item_id: String,
    },
    Content {
        vault_id: String,
        item_id: String,
        field_id: String,
    },
}
