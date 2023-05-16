use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TailscaleWebhookError {
    #[error("webhook has no signature")]
    NotSigned,
    #[error("webhook has an invalid signature (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("Tailscale-Webhook-Signature header is empty")]
    EmptyHeader,
    #[error("incorrect unix timestamp ({found:?})")]
    IncorrectTimestamp { found: String },
}

impl From<ParseIntError> for TailscaleWebhookError {
    fn from(error: ParseIntError) -> Self {
        Self::IncorrectTimestamp {
            found: error.to_string(),
        }
    }
}
