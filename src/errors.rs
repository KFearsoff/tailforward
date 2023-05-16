use thiserror::Error;

#[derive(Error, Debug)]
pub enum TailscaleWebhookError {
    #[error("webhook has no signature")]
    NotSigned,
    #[error("webhook has an invalid signature (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
}
