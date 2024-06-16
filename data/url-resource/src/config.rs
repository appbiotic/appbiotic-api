use std::time::Duration;

use url::Url;

// NOTE: Could add retry logic before caching a failure result.

#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub struct UrlResource {
    pub url: Url,

    /// A resource will be refetched if the cached values age is greater than
    /// [UrlResource::cache_ttl] if set.
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            deserialize_with = "duration_str::deserialize_option_duration"
        )
    )]
    pub cache_ttl: Option<Duration>,

    #[cfg_attr(feature = "serde", serde(flatten))]
    pub provider: UrlResourceProvider,
}

#[derive(Debug, strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(strum::AsRefStr, strum::Display))]
#[strum_discriminants(name(UrlResourceProviderKind))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
#[non_exhaustive]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub enum UrlResourceProvider {
    Tokio(TokioUrlResourceProvider),
}

#[derive(Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub struct TokioUrlResourceProvider {
    pub mpsc_channel_size: Option<usize>,
}
