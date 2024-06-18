use derive_new::new;

#[derive(Clone, new, thiserror::Error, Debug)]
pub enum JwtDecoderError {
    #[error("JWT header parsing failed: {message}")]
    HeaderParsingFailed { message: String },
    #[error("Internal error: {message}")]
    InternalError { message: String },
    #[error("JWKS fetch error: {message}")]
    JwksFetchError { message: String },
    #[error("Failed precondition: {message}")]
    FailedPrecondition { message: String },
    #[error("JWT was missing key ID `kid` option")]
    MissingKeyId,
    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },
    #[error("Unsupported JWK for kid `{kid}`: {message}")]
    UnsupportedJwk { kid: String, message: String },
    #[error("Validation failed: {message}")]
    ValidationFailed { message: String },
}
