use derive_new::new;

#[derive(new, thiserror::Error, Debug)]
pub enum OnePasswordError {
    #[error("Resource parsing failed")]
    ResourceParsingFailed,
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Unknown: {message}")]
    Unknown { message: String },
}
