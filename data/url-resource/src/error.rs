use crate::config::UrlResourceProviderKind;

#[derive(
    Clone, Debug, thiserror::Error, strum_macros::EnumDiscriminants, derive_new::new, Eq, PartialEq,
)]
#[strum_discriminants(derive(strum::AsRefStr))]
#[strum_discriminants(name(UrlResourceErrorReason))]
#[strum_discriminants(strum(serialize_all = "SCREAMING_SNAKE_CASE"))]
pub enum UrlResourceError {
    #[error("Failed precondition: {message}")]
    FailedPrecondition { message: String },
    #[error("Denied access to resource: {message}")]
    ResourceAccessDenied { message: String },
    #[error("Resource not found: {message}")]
    ResourceNotFound { message: String },
    #[error("Error resource: {message}")]
    ResourceReadError { message: String },
    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },
    #[error("Unknown: {message}")]
    Unknown { message: String },
    #[error("Unsupported UrlResourceProvider kind `{kind}`")]
    UnsupportedProvider { kind: UrlResourceProviderKind },
    #[error("URL scheme `{scheme}` not supported")]
    UnsupportedScheme { scheme: String },
}

impl UrlResourceError {
    pub const DOMAIN: &'static str = "com.appbiotic.data.url-resource";
}

#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub struct UrlResourceErrorReports {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub errors: Vec<UrlResourceErrorReport>,
}

impl From<UrlResourceErrorReport> for UrlResourceErrorReports {
    fn from(value: UrlResourceErrorReport) -> Self {
        Self {
            errors: vec![value],
        }
    }
}

#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub struct UrlResourceErrorReport {
    pub code: String,
    pub message: String,
}

impl From<UrlResourceError> for UrlResourceErrorReport {
    fn from(value: UrlResourceError) -> Self {
        Self {
            code: UrlResourceErrorReason::from(&value).as_ref().to_owned(),
            message: value.to_string(),
        }
    }
}
