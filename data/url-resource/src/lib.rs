//! A library for fetching resources from URLs and various ways of caching and serving them.

use async_trait::async_trait;

pub mod config;
pub mod error;
#[cfg(feature = "tokio")]
pub mod tokio;

use bytes::Bytes;
use error::UrlResourceError;

/// NOTE: Could add a streaming method.
#[async_trait]
pub trait UrlResourceFetch {
    async fn fetch(&self) -> Result<UrlResourceContent, UrlResourceError>;
}

#[derive(Clone)]
pub struct UrlResourceContent {
    pub data: Bytes,
    pub hash: Option<String>,
}
