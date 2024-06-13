use std::{
    collections::HashMap, convert::AsRef, os::unix::fs::MetadataExt, path::Path, sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use cached::{Cached, CanExpire, TimedCache};
use futures::future::join_all;
use jsonwebtoken::{
    decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, TokenData, Validation,
};
use tokio::{
    fs::File,
    io::AsyncReadExt,
    sync::Mutex,
    time::{timeout, Instant},
};
use tracing::{info_span, warn, Instrument};
use url::Url;

use crate::{config, error::JwtDecoderError, JwtDecode};

pub struct JwtDecoder {
    jwks_urls: Vec<Url>,
    url_to_jwks: Arc<Mutex<HashMap<Url, JwksResult>>>,
    kid_to_jwk: Arc<Mutex<TimedCache<String, JwkEntry>>>,
    max_wait: Duration,
    ttl: Duration,
    validation: Validation,
}

#[derive(Clone)]
struct JwksResult {
    jwks: Result<JwkSet, JwtDecoderError>,
    expiration: Instant,
}

#[derive(Clone)]
struct JwkEntry {
    jwk: Arc<DecodingKey>,
    expiration: Instant,
}

impl CanExpire for JwkEntry {
    fn is_expired(&self) -> bool {
        self.expiration < Instant::now()
    }
}

impl JwtDecoder {
    pub fn new(config: config::JwtDecoder) -> Result<Self, JwtDecoderError> {
        let config::JwtDecoderKind::Tokio(_) = config.kind;

        let validation = {
            // NOTE: algorithm in `new` will be overwritten.
            let mut validation = Validation::new(Algorithm::RS256);
            if !config.algorithms.is_empty() {
                validation.algorithms = config.algorithms;
            }
            if !config.required_spec_claims.is_empty() {
                validation.set_required_spec_claims(&config.required_spec_claims);
            }
            if !config.valid_audiences.is_empty() {
                validation.set_audience(&config.valid_audiences);
            }
            if !config.valid_issuers.is_empty() {
                validation.set_issuer(&config.valid_issuers);
            }
            validation
        };

        let ttl = config.jwks_ttl.unwrap_or(Duration::from_secs(60));

        Ok(Self {
            jwks_urls: config.jwks_urls,
            url_to_jwks: Default::default(),
            kid_to_jwk: Arc::new(Mutex::new(TimedCache::with_lifespan(ttl.as_secs()))),
            ttl,
            max_wait: config.jwks_max_wait.unwrap_or(Duration::from_secs(1)),
            validation,
        })
    }

    async fn jwk(&self, kid: &str) -> Result<Option<Arc<DecodingKey>>, JwtDecoderError> {
        // Immediately return an unexpired JWK if available.
        if let Some(jwk_entry) = self.kid_to_jwk.lock().await.cache_get(kid) {
            return Ok(Some(jwk_entry.jwk.clone()));
        }

        // Otherwise, refresh all expired.
        self.refresh_all().await;

        // Then, just return whatever is found.
        Ok(self
            .kid_to_jwk
            .lock()
            .await
            .cache_get(kid)
            .map(|e| e.jwk.clone()))
    }

    async fn fetch_file(path: impl AsRef<Path>) -> Result<JwkSet, JwtDecoderError> {
        let mut file = File::open(path)
            .await
            .map_err(|err| JwtDecoderError::new_jwks_fetch_error(err.to_string()))?;
        let metadata = file
            .metadata()
            .await
            .map_err(|err| JwtDecoderError::new_jwks_fetch_error(err.to_string()))?;
        let size = usize::try_from(metadata.size())
            .map_err(|err| JwtDecoderError::new_jwks_fetch_error(err.to_string()))?;
        let mut data: Vec<u8> = Vec::with_capacity(size);
        file.read_to_end(&mut data)
            .await
            .map_err(|err| JwtDecoderError::new_jwks_fetch_error(err.to_string()))?;
        serde_json::from_slice(&data)
            .map_err(|err| JwtDecoderError::new_jwks_fetch_error(err.to_string()))
    }

    async fn fetch_jwks<'a>(
        url: &'a Url,
        max_wait: &Duration,
        ttl: &Duration,
    ) -> (&'a Url, JwksResult) {
        let result = timeout(
            *max_wait,
            async move {
                match url.scheme() {
                    "file" => match url.to_file_path() {
                        Ok(path) => Self::fetch_file(path).await,
                        Err(_) => Err(JwtDecoderError::new_jwks_fetch_error(
                            "Failed to extract path from file URL".to_owned(),
                        )),
                    },
                    scheme => Err(JwtDecoderError::new_jwks_fetch_error(format!(
                        "Unsupported JWKS URL scheme `{scheme}`"
                    ))),
                }
            }
            .instrument(info_span!("fetch_jwks", url = url.as_str())),
        )
        .await
        .unwrap_or_else(|_| {
            Err(JwtDecoderError::new_jwks_fetch_error(
                "timed out".to_owned(),
            ))
        });

        (
            url,
            JwksResult {
                jwks: result,
                expiration: Instant::now() + *ttl,
            },
        )
    }

    async fn refresh_all(&self) {
        let mut url_to_jwks = self.url_to_jwks.lock().await;
        let now = Instant::now();

        let mut fetches = Vec::new();
        for jwks_url in &self.jwks_urls {
            if let Some(entry) = url_to_jwks.get(jwks_url) {
                if entry.expiration < now {
                    fetches.push(Self::fetch_jwks(jwks_url, &self.max_wait, &self.ttl));
                }
            } else {
                fetches.push(Self::fetch_jwks(jwks_url, &self.max_wait, &self.ttl));
            }
        }

        if fetches.is_empty() {
            return;
        }

        let results = join_all(fetches).await;

        // Apply results to cache.

        for result in results {
            let url = result.0;
            let result = result.1;

            if let Some(existing) = url_to_jwks.get_mut(url) {
                *existing = result;
            } else {
                url_to_jwks.insert(url.to_owned(), result);
            }
        }

        // Apply cache in order of URLs with descending priority.

        let mut kid_to_jwk = self.kid_to_jwk.lock().await;
        kid_to_jwk.cache_clear();
        for url in &self.jwks_urls {
            if let Some(result) = url_to_jwks.get(url) {
                if let Ok(jwks) = &result.jwks {
                    for key in &jwks.keys {
                        if let Some(kid) = &key.common.key_id {
                            // Do not overwrite JWK with lower priority URL JWK
                            if kid_to_jwk.cache_get(kid).is_some() {
                                continue;
                            }
                            match DecodingKey::from_jwk(key) {
                                Ok(decoding_key) => {
                                    kid_to_jwk.cache_set(
                                        kid.to_owned(),
                                        JwkEntry {
                                            jwk: Arc::new(decoding_key),
                                            expiration: result.expiration,
                                        },
                                    );
                                }
                                Err(err) => {
                                    warn!(?url, kid, error = ?err, "Failed to create decoding key from JWK");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl JwtDecode for JwtDecoder {
    async fn decode(&self, token: &str) -> Result<TokenData<serde_json::Value>, JwtDecoderError> {
        let header = decode_header(token)
            .map_err(|err| JwtDecoderError::new_header_parsing_failed(err.to_string()))?;
        let kid = header
            .kid
            .as_deref()
            .ok_or(JwtDecoderError::new_validation_failed(
                "Header missing kid".to_owned(),
            ))?;
        let key = self
            .jwk(kid)
            .await?
            .ok_or(JwtDecoderError::new_missing_key_id())?;
        let claims: TokenData<serde_json::Value> = decode(token, &key, &self.validation)
            .map_err(|err| JwtDecoderError::new_validation_failed(err.to_string()))?;
        Ok(claims)
    }
}

#[cfg(test)]
mod test {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
    use jsonwebtoken::{
        decode, encode,
        jwk::{
            AlgorithmParameters, CommonParameters, Jwk, JwkSet, KeyAlgorithm, PublicKeyUse,
            RSAKeyParameters,
        },
        Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
    };
    use rsa::{
        pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
        traits::PublicKeyParts,
        RsaPrivateKey, RsaPublicKey,
    };
    use tokio::{fs::File, io::AsyncWriteExt};
    use tracing_test::traced_test;
    use url::Url;

    use crate::{config, tokio::JwtDecoder, JwtDecode};

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Claims {
        aud: String,
        sub: String,
        exp: u64,
    }

    #[traced_test]
    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn it_works() {
        // Create key pair
        // NOTE: Could check in generated key for use in testing to speed this up

        let kid = "my-key".to_owned();
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);

        let jwk = Jwk {
            common: CommonParameters {
                public_key_use: Some(PublicKeyUse::Signature),
                key_operations: None,
                key_algorithm: Some(KeyAlgorithm::RS256),
                key_id: Some(kid.to_owned()),
                x509_url: None,
                x509_chain: None,
                x509_sha1_fingerprint: None,
                x509_sha256_fingerprint: None,
            },
            algorithm: AlgorithmParameters::RSA(RSAKeyParameters {
                key_type: jsonwebtoken::jwk::RSAKeyType::RSA,
                n: URL_SAFE_NO_PAD.encode(pub_key.n().to_bytes_be()),
                e: URL_SAFE_NO_PAD.encode(pub_key.e().to_bytes_be()),
            }),
        };

        let aud = "some-users".to_owned();

        // Create JWT

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_owned());

        let encoding_key = EncodingKey::from_rsa_der(priv_key.to_pkcs1_der().unwrap().as_bytes());
        let jwt = encode(
            &header,
            &Claims {
                aud: aud.to_owned(),
                sub: "user@example.com".to_owned(),
                exp: (SystemTime::now() + Duration::from_secs(30))
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
            &encoding_key,
        )
        .expect("Failed to encode JWT");

        let bad_aud_jwt = encode(
            &header,
            &Claims {
                aud: "nobody".to_owned(),
                sub: "user@example.com".to_owned(),
                exp: (SystemTime::now() + Duration::from_secs(30))
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
            &encoding_key,
        )
        .expect("Failed to encode JWT");

        // Double check that JWT validation should work for both DER and JWK derived keys.

        let validation = {
            let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
            validation.set_audience(&[aud.to_owned()]);
            validation.set_required_spec_claims(&["exp", "sub", "aud"]);
            validation
        };

        let decoding_key = DecodingKey::from_rsa_der(pub_key.to_pkcs1_der().unwrap().as_bytes());
        let token_data: TokenData<Claims> =
            decode(&jwt, &decoding_key, &validation).expect("decoded jwt from key by der");
        assert_eq!(token_data.claims.sub, "user@example.com");

        let decoding_key_from_jwk = DecodingKey::from_jwk(&jwk).unwrap();
        let token_data: TokenData<Claims> = decode(&jwt, &decoding_key_from_jwk, &validation)
            .expect("decoded jwt from key by jwk parts");
        assert_eq!(token_data.claims.sub, "user@example.com");

        // Test core JwtDecoder workflow

        let jwks = JwkSet { keys: vec![jwk] };
        let jwks_string = serde_json::to_string_pretty(&jwks).unwrap();

        let temp_dir = tempfile::TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("my-key.jwks");
        let mut file = File::create_new(&temp_file).await.unwrap();
        file.write_all(jwks_string.as_bytes()).await.unwrap();

        let config = config::JwtDecoder {
            jwks_urls: vec![Url::from_file_path(&temp_file).unwrap()],
            algorithms: vec![Algorithm::RS256],
            required_spec_claims: vec!["aud".to_owned(), "sub".to_owned(), "exp".to_owned()],
            valid_audiences: vec!["some-users".to_owned()],
            valid_issuers: vec!["an-issuer".to_owned()],
            jwks_max_wait: None,
            jwks_ttl: None,
            kind: config::JwtDecoderKind::Tokio(config::TokioJwtDecoder {}),
        };

        let jwt_decoder = JwtDecoder::new(config).unwrap();

        let token_data = jwt_decoder.decode(&jwt).await.unwrap();
        assert_eq!(
            token_data.claims.get("sub").map(|v| v.as_str()).unwrap(),
            Some("user@example.com")
        );

        assert!(jwt_decoder.decode(&bad_aud_jwt).await.is_err());
    }
}
