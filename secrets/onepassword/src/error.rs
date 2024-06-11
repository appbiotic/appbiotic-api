use derive_new::new;

#[derive(new, thiserror::Error, Debug)]
pub enum OnePasswordError {
    #[error("Unknown: {message}")]
    Unknown { message: String },
}
