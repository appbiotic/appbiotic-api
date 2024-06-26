mod error;
mod model;

pub use error::*;
pub use model::*;

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait OnePassword {
    async fn api_version(
        &self,
        request: ApiVersionRequest,
    ) -> Result<ApiVersionResponse, OnePasswordError>;
    async fn document_create(
        &self,
        request: DocumentCreateRequest,
    ) -> Result<DocumentCreateResponse, OnePasswordError>;
    async fn item_get(&self, request: ItemGetRequest) -> Result<ItemGetResponse, OnePasswordError>;
    async fn read(&self, request: ReadRequest) -> Result<ReadResponse, OnePasswordError>;
    async fn user_get(&self, request: UserGetRequest) -> Result<UserGetResponse, OnePasswordError>;
}
