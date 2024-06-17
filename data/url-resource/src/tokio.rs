use std::{borrow::BorrowMut, ops::Deref, os::unix::fs::MetadataExt, time::Duration};

use async_trait::async_trait;
use bytes::Bytes;
use tokio::{
    fs::File,
    io::AsyncReadExt,
    sync::{mpsc, oneshot, watch},
    time::{sleep_until, Instant},
};
use tokio_stream::{wrappers::WatchStream, StreamExt};
use tracing::{debug, error, info_span, trace, Instrument};
use url::Url;

use crate::{config, error::UrlResourceError, UrlResourceContent, UrlResourceFetch};

const DEFAULT_MPSC_CHANNEL_SIZE: usize = 8;

#[derive(strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(strum::AsRefStr))]
#[strum_discriminants(name(UrlResourceCommandType))]
#[strum_discriminants(strum(serialize_all = "SCREAMING_SNAKE_CASE"))]
enum UrlResourceCommand {
    Fetch {
        respond_to: oneshot::Sender<watch::Receiver<FetchStatus>>,
    },
    Clear,
}

#[derive(Clone, strum_macros::EnumDiscriminants)]
#[strum_discriminants(derive(strum::AsRefStr))]
#[strum_discriminants(name(FetchStatusType))]
#[strum_discriminants(strum(serialize_all = "SCREAMING_SNAKE_CASE"))]
enum FetchStatus {
    NotFetched,
    Fetching,
    Fetched {
        result: Result<UrlResourceContent, UrlResourceError>,
        expiration: Instant,
    },
}

#[derive(Clone)]
pub struct UrlResource {
    commands_tx: mpsc::Sender<UrlResourceCommand>,
    url: Url,
}

impl UrlResource {
    pub fn new(config: config::UrlResource) -> Result<Self, UrlResourceError> {
        #[allow(unreachable_patterns)]
        let tokio_config = match config.provider {
            config::UrlResourceProvider::Tokio(tokio_provider) => tokio_provider,
            other_provider => {
                return Err(UrlResourceError::new_unsupported_provider(
                    config::UrlResourceProviderKind::from(other_provider),
                ))
            }
        };

        let (commands_tx, commands_rx) = mpsc::channel(
            tokio_config
                .mpsc_channel_size
                .unwrap_or(DEFAULT_MPSC_CHANNEL_SIZE),
        );

        let (watch_tx, watch_rx) = watch::channel(FetchStatus::NotFetched);

        let actor = UrlResourceActor {
            url: config.url.clone(),
            #[cfg(feature = "sha256")]
            hash: config.hash,
            commands_tx: commands_tx.clone(),
            commands_rx,
            watch_tx,
            watch_rx,
            ttl: config.cache_ttl.unwrap_or(Duration::from_secs(15 * 60)),
        };

        let actor = tokio::spawn(actor.run());

        let cloned_url = config.url.clone();
        tokio::spawn(async move {
            if let Err(err) = actor.await {
                error!(
                    url = cloned_url.to_string(),
                    error = err.to_string(),
                    "UrlResourceActor for url finished with error",
                );
            }
        });

        Ok(Self {
            commands_tx,
            url: config.url,
        })
    }
}

#[async_trait]
impl UrlResourceFetch for UrlResource {
    async fn fetch(&self) -> Result<UrlResourceContent, UrlResourceError> {
        async move {
            let (respond_to, result) = oneshot::channel();
            trace!(url = self.url.as_str(), "Sending UrlResourceCommand::Fetch");
            self.commands_tx
                .send(UrlResourceCommand::Fetch { respond_to })
                .await
                .map_err(UrlResourceError::from)?;
            let mut watch_stream = WatchStream::new(result.await.map_err(UrlResourceError::from)?);
            while let Some(event) = watch_stream.next().await {
                match event {
                    FetchStatus::Fetched {
                        result,
                        expiration: _expiration,
                    } => {
                        trace!(
                            url = self.url.as_str(),
                            ok = result.is_ok(),
                            "Received FetchStatus::Fetched"
                        );
                        return result;
                    }
                    FetchStatus::Fetching => {
                        trace!(url = self.url.as_str(), "Received FetchStatus::Fetching");
                        continue;
                    }
                    FetchStatus::NotFetched => {
                        trace!(url = self.url.as_str(), "Received FetchStatus::NotFetched");
                        continue;
                    }
                }
            }

            Err(UrlResourceError::new_service_unavailable(
                "URL resource watch stream ended".to_owned(),
            ))
        }
        .instrument(info_span!("fetch"))
        .await
    }
}

struct UrlResourceActor {
    url: Url,
    #[cfg(feature = "sha256")]
    hash: Option<config::UrlResourceHash>,
    commands_tx: mpsc::Sender<UrlResourceCommand>,
    commands_rx: mpsc::Receiver<UrlResourceCommand>,
    watch_tx: watch::Sender<FetchStatus>,
    watch_rx: watch::Receiver<FetchStatus>,
    ttl: Duration,
}

impl UrlResourceActor {
    async fn run(mut self) {
        while let Some(command) = self.commands_rx.recv().await {
            let state = FetchStatusType::from(self.watch_tx.borrow().deref());
            let command_type = UrlResourceCommandType::from(&command);

            trace!(
                url = self.url.as_str(),
                state = state.as_ref(),
                command = command_type.as_ref(),
                "Processing command"
            );

            match command {
                UrlResourceCommand::Clear => {
                    let needs_clear = match self.watch_tx.borrow().deref() {
                        FetchStatus::Fetched {
                            result: _,
                            expiration,
                        } => {
                            let now = Instant::now();
                            let expiration = *expiration;
                            expiration < now
                        }
                        _ => false,
                    };

                    if needs_clear {
                        trace!(
                            url = self.url.as_str(),
                            state = state.as_ref(),
                            command = command_type.as_ref(),
                            needs_clear,
                            "Clearing cache"
                        );
                        let _ = self.watch_tx.send(FetchStatus::NotFetched);
                    }
                }
                UrlResourceCommand::Fetch { respond_to } => {
                    let watch_tx = self.watch_tx.borrow_mut();

                    match watch_tx.borrow().deref() {
                        FetchStatus::NotFetched => {
                            let _ = respond_to.send(self.watch_rx.clone());
                        }
                        FetchStatus::Fetching => {
                            let _ = respond_to.send(self.watch_rx.clone());
                            continue;
                        }
                        FetchStatus::Fetched {
                            result: _,
                            expiration,
                        } => {
                            let now = Instant::now();
                            let expiration = *expiration;
                            let expired = expiration < now;
                            trace!(
                                url = self.url.as_str(),
                                state = state.as_ref(),
                                command = command_type.as_ref(),
                                expired,
                                "Processing fetch"
                            );
                            if expired {
                                // Need to fetch a new value.
                                let _ = watch_tx.send(FetchStatus::Fetching);
                            } else {
                                // Return the existing value.
                                let _ = respond_to.send(self.watch_rx.clone());
                                continue;
                            }
                        }
                    }

                    let url = self.url.clone();
                    #[cfg(feature = "sha256")]
                    let hash = self.hash.clone();
                    let watch_tx = self.watch_tx.clone();
                    let commands_tx = self.commands_tx.clone();
                    let ttl = self.ttl;
                    tokio::spawn(async move {
                        let tracing_url = url.clone();
                        let scheme = tracing_url.scheme();
                        let span = info_span!("fetch_url", scheme, url = tracing_url.as_str());

                        let result = fetch_url(
                            url,
                            #[cfg(feature = "sha256")]
                            hash,
                        )
                        .instrument(span)
                        .await
                        .inspect_err(|err| {
                            error!(
                                error = err.to_string(),
                                url = tracing_url.as_str(),
                                "Failed to fetch URL"
                            )
                        });

                        let now = Instant::now();
                        let expiration = now + ttl;

                        debug!(
                            url = tracing_url.as_str(),
                            ttl_seconds = ttl.as_secs(),
                            "Fetched URL"
                        );
                        let _ = watch_tx.send(FetchStatus::Fetched { result, expiration });
                        sleep_until(expiration).await;

                        debug!(
                            url = tracing_url.as_str(),
                            ttl_seconds = ttl.as_secs(),
                            "TTL reached clearing cache"
                        );
                        let _ = commands_tx.send(UrlResourceCommand::Clear).await;
                    });
                }
            }
        }
        trace!(
            url = self.url.as_str(),
            "UrlResourceActor command_rx queue closed"
        );
    }
}

async fn fetch_url(
    url: Url,
    #[cfg(feature = "sha256")] hash: Option<config::UrlResourceHash>,
) -> Result<UrlResourceContent, UrlResourceError> {
    let data = match url.scheme() {
        "file" => {
            let file_path = url.to_file_path().map_err(|_| {
                UrlResourceError::new_failed_precondition(format!(
                    "Configured URL resource invalid: {url}"
                ))
            })?;

            let mut file = File::open(&file_path)
                .await
                .map_err(|err| UrlResourceError::new_resource_not_found(err.to_string()))?;
            let metadata = file
                .metadata()
                .await
                .map_err(|err| UrlResourceError::new_resource_read_error(err.to_string()))?;
            let size = usize::try_from(metadata.size()).map_err(|_| {
                UrlResourceError::new_resource_read_error(format!(
                    "Cannot read resource since size `{}` greater than usize max `{}`",
                    metadata.size(),
                    usize::MAX
                ))
            })?;
            let mut data: Vec<u8> = Vec::with_capacity(size);
            file.read_to_end(&mut data)
                .await
                .map_err(|err| UrlResourceError::new_resource_read_error(err.to_string()))?;

            data
        }
        scheme => return Err(UrlResourceError::new_unsupported_scheme(scheme.to_owned())),
    };

    // NOTE: Could be doing hash in parallel with download.
    #[cfg(feature = "sha256")]
    let hash = hash.map(|hash| match hash {
        config::UrlResourceHash::Sha256 => format!("sha256:{}", sha256::digest(&data)),
    });

    Ok(UrlResourceContent {
        data: Bytes::from(data),
        #[cfg(feature = "sha256")]
        hash,
    })
}

impl<T> From<mpsc::error::SendError<T>> for UrlResourceError {
    fn from(value: mpsc::error::SendError<T>) -> Self {
        UrlResourceError::new_service_unavailable(value.to_string())
    }
}

impl From<watch::error::RecvError> for UrlResourceError {
    fn from(value: watch::error::RecvError) -> Self {
        UrlResourceError::new_service_unavailable(value.to_string())
    }
}

impl From<oneshot::error::RecvError> for UrlResourceError {
    fn from(value: oneshot::error::RecvError) -> Self {
        UrlResourceError::new_service_unavailable(value.to_string())
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::{fs::write, time};
    use tracing_test::traced_test;
    use url::Url;

    use crate::{
        config::{self, TokioUrlResourceProvider},
        error::{UrlResourceError, UrlResourceErrorReason},
        UrlResourceFetch,
    };

    use super::UrlResource;

    // TODO: Test cache ttl with https://docs.rs/tokio/latest/tokio/time/fn.pause.html

    #[tokio::test]
    async fn ftp_unsupported() {
        let config = config::UrlResource {
            url: Url::parse("ftp://example.com/hello.txt").unwrap(),
            cache_ttl: Some(Duration::from_secs(60)),
            hash: None,
            provider: config::UrlResourceProvider::Tokio(TokioUrlResourceProvider {
                mpsc_channel_size: None,
            }),
        };

        let url_resource = UrlResource::new(config).unwrap();
        let result = url_resource.fetch().await;
        let err = result.err().unwrap();

        assert_eq!(
            err,
            UrlResourceError::new_unsupported_scheme("ftp".to_owned()),
        );
    }

    #[tokio::test]
    async fn file_resource() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("hello.txt");

        let config = config::UrlResource {
            url: Url::from_file_path(&file_path).unwrap(),
            cache_ttl: Some(Duration::from_secs(60)),
            hash: Some(config::UrlResourceHash::Sha256),
            provider: config::UrlResourceProvider::Tokio(TokioUrlResourceProvider {
                mpsc_channel_size: None,
            }),
        };

        let url_resource = UrlResource::new(config).unwrap();

        let file_content = "Hello, world!\n";
        let expected_hash =
            "sha256:d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5";

        write(&file_path, file_content.as_bytes()).await.unwrap();

        let result = url_resource.fetch().await;
        let content = result.ok().unwrap();

        assert_eq!(
            String::from_utf8(content.data.to_vec()).unwrap(),
            file_content
        );
        assert_eq!(content.hash.as_deref(), Some(expected_hash));
    }

    #[traced_test]
    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn ttl_works() {
        let cache_ttl_millis = Duration::from_millis(1000);

        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("hello.txt");

        let config = config::UrlResource {
            url: Url::from_file_path(&file_path).unwrap(),
            cache_ttl: Some(cache_ttl_millis.to_owned()),
            hash: None,
            provider: config::UrlResourceProvider::Tokio(TokioUrlResourceProvider {
                mpsc_channel_size: None,
            }),
        };

        let url_resource = UrlResource::new(config).unwrap();

        let result = url_resource.fetch().await;
        assert_eq!(
            result.err().map(|e| UrlResourceErrorReason::from(&e)),
            Some(UrlResourceErrorReason::ResourceNotFound),
            "testing that resource is initially not found"
        );

        time::advance(cache_ttl_millis - Duration::from_millis(10)).await;

        let result = url_resource.fetch().await;
        assert_eq!(
            result.err().map(|e| UrlResourceErrorReason::from(&e)),
            Some(UrlResourceErrorReason::ResourceNotFound),
            "testing that resource not found is still cached"
        );

        // Advance time past ttl so that cache gets cleared.
        time::advance(Duration::from_millis(11)).await;

        let file_content = "Hello, world!\n";

        write(&file_path, file_content.as_bytes()).await.unwrap();

        // Expecting that new fetch will trigger an a real fetch since cache was cleared.
        let result = url_resource.fetch().await;
        let content = result.ok().unwrap();

        assert_eq!(
            String::from_utf8(content.data.to_vec()).unwrap(),
            file_content
        );
    }
}
