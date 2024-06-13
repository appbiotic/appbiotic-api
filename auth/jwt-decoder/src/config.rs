use std::time::Duration;

use jsonwebtoken::Algorithm;
use serde_with::{serde_as, DurationSecondsWithFrac};
use url::Url;

#[serde_as]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct JwtDecoder {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jwks_urls: Vec<Url>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub algorithms: Vec<Algorithm>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_spec_claims: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub valid_audiences: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub valid_issuers: Vec<String>,

    #[serde_as(as = "Option<DurationSecondsWithFrac<f64>>")]
    #[serde(rename = "jwks_max_wait_sec")]
    pub jwks_max_wait: Option<Duration>,

    #[serde_as(as = "Option<DurationSecondsWithFrac<f64>>")]
    #[serde(rename = "jwks_ttl_sec")]
    pub jwks_ttl: Option<Duration>,

    #[serde(flatten)]
    pub kind: JwtDecoderKind,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, strum::AsRefStr)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum JwtDecoderKind {
    Tokio(TokioJwtDecoder),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TokioJwtDecoder {}
