mod error;
mod model;

pub use error::*;
pub use model::*;

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait OnePassword {
    async fn api_version(&self) -> Result<ApiVersion, OnePasswordError>;
    async fn item_get(&self, vault_id: String, item_id: String) -> Result<Item, OnePasswordError>;
    async fn user_get_me(&self) -> Result<User, OnePasswordError>;
}
