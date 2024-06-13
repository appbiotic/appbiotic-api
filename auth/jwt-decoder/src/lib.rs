use async_trait::async_trait;

pub mod config;
pub mod error;
pub mod tokio;

use error::JwtDecoderError;
use jsonwebtoken::TokenData;

#[async_trait]
pub trait JwtDecode {
    async fn decode(&self, token: &str) -> Result<TokenData<serde_json::Value>, JwtDecoderError>;
}
