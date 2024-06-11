use appbiotic_api_secrets_onepassword::{
    ApiVersionRequest, ApiVersionResponse, ItemGetRequest, ItemGetResponse, OnePassword,
    OnePasswordError, ReadRequest, ReadResponse, UserGetRequest, UserGetResponse,
};
use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tracing::{info_span, Instrument};

#[derive(Clone)]
pub struct OnePasswordClientHandle {
    commands_tx: mpsc::Sender<OnePasswordCommand>,
}

#[async_trait]
impl OnePassword for OnePasswordClientHandle {
    async fn api_version(
        &self,
        request: ApiVersionRequest,
    ) -> Result<ApiVersionResponse, OnePasswordError> {
        let (respond_to, mut response) = oneshot::channel();
        let command = OnePasswordCommand::ApiVersion {
            request,
            respond_to,
        };
        async move {
            let _ = self
                .commands_tx
                .send(command)
                .await
                .map_err(SendError::from)?;
            response.try_recv().map_err(RecvError::from)?
        }
        .instrument(info_span!("api_version"))
        .await
        .map_err(OnePasswordError::into)
    }

    async fn item_get(&self, request: ItemGetRequest) -> Result<ItemGetResponse, OnePasswordError> {
        let (respond_to, mut response) = oneshot::channel();
        let command = OnePasswordCommand::ItemGet {
            request,
            respond_to,
        };
        async move {
            let _ = self
                .commands_tx
                .send(command)
                .await
                .map_err(SendError::from)?;
            response.try_recv().map_err(RecvError::from)?
        }
        .instrument(info_span!("item_get"))
        .await
        .map_err(OnePasswordError::into)
    }

    async fn read(&self, request: ReadRequest) -> Result<ReadResponse, OnePasswordError> {
        let (respond_to, mut response) = oneshot::channel();
        let command = OnePasswordCommand::Read {
            request,
            respond_to,
        };
        async move {
            let _ = self
                .commands_tx
                .send(command)
                .await
                .map_err(SendError::from)?;
            response.try_recv().map_err(RecvError::from)?
        }
        .instrument(info_span!("read"))
        .await
        .map_err(OnePasswordError::into)
    }

    async fn user_get(&self, request: UserGetRequest) -> Result<UserGetResponse, OnePasswordError> {
        let (respond_to, mut response) = oneshot::channel();
        let command = OnePasswordCommand::UserGet {
            request,
            respond_to,
        };
        async move {
            let _ = self
                .commands_tx
                .send(command)
                .await
                .map_err(SendError::from)?;
            response.try_recv().map_err(RecvError::from)?
        }
        .instrument(info_span!("user_get"))
        .await
        .map_err(OnePasswordError::into)
    }
}

enum OnePasswordCommand {
    ApiVersion {
        request: ApiVersionRequest,
        respond_to: oneshot::Sender<Result<ApiVersionResponse, OnePasswordError>>,
    },
    ItemGet {
        request: ItemGetRequest,
        respond_to: oneshot::Sender<Result<ItemGetResponse, OnePasswordError>>,
    },
    Read {
        request: ReadRequest,
        respond_to: oneshot::Sender<Result<ReadResponse, OnePasswordError>>,
    },
    UserGet {
        request: UserGetRequest,
        respond_to: oneshot::Sender<Result<UserGetResponse, OnePasswordError>>,
    },
}

pub struct OnePasswordClient {
    client: super::OnePasswordClient,
    commands_rx: mpsc::Receiver<OnePasswordCommand>,
}

impl OnePasswordClient {
    pub fn start(client: super::OnePasswordClient, channel_size: usize) -> OnePasswordClientHandle {
        let (commands_tx, commands_rx) = mpsc::channel(channel_size);
        let actor = OnePasswordClient {
            client,
            commands_rx,
        };
        tokio::spawn(actor.run());
        OnePasswordClientHandle { commands_tx }
    }

    async fn run(mut self) {
        while let Some(command) = self.commands_rx.recv().await {
            match command {
                OnePasswordCommand::ApiVersion {
                    request,
                    respond_to,
                } => {
                    let _ = respond_to.send(self.client.api_version(request).await);
                }
                OnePasswordCommand::ItemGet {
                    request,
                    respond_to,
                } => {
                    let _ = respond_to.send(self.client.item_get(request).await);
                }
                OnePasswordCommand::Read {
                    request,
                    respond_to,
                } => {
                    let _ = respond_to.send(self.client.read(request).await);
                }
                OnePasswordCommand::UserGet {
                    request,
                    respond_to,
                } => {
                    let _ = respond_to.send(self.client.user_get(request).await);
                }
            }
        }
    }
}

struct SendError<T>(mpsc::error::SendError<T>);

impl<T> From<mpsc::error::SendError<T>> for SendError<T> {
    fn from(value: mpsc::error::SendError<T>) -> Self {
        Self(value)
    }
}

impl<T> From<SendError<T>> for OnePasswordError {
    fn from(_value: SendError<T>) -> Self {
        Self::ServiceUnavailable
    }
}

struct RecvError;

impl From<oneshot::error::TryRecvError> for RecvError {
    fn from(_value: oneshot::error::TryRecvError) -> Self {
        Self
    }
}

impl From<RecvError> for OnePasswordError {
    fn from(_value: RecvError) -> Self {
        Self::ServiceUnavailable
    }
}
