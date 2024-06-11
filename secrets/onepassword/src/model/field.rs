use crate::{FieldPurpose, FieldSection, FieldType, GeneratorRecipe, PasswordDetails};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub struct Field {
    pub id: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub section: Option<FieldSection>,

    #[cfg_attr(
        feature = "serde",
        serde(default, rename = "type", skip_serializing_if = "Option::is_none")
    )]
    pub field_type: Option<FieldType>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub purpose: Option<FieldPurpose>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub label: Option<String>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub value: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "std::ops::Not::not")
    )]
    pub generate: bool,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub recipe: Option<GeneratorRecipe>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub password_details: Option<PasswordDetails>,

    pub reference: String,
}
