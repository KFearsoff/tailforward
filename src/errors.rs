use serde_json::error::Error as SerdeJsonError;
use std::io;
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
    #[error("error while serializing/deserializing: {error}")]
    Serde { error: String },
    #[error("error reading the file: {error}")]
    IoError { error: String },
}

impl From<ParseIntError> for TailscaleWebhookError {
    fn from(error: ParseIntError) -> Self {
        Self::IncorrectTimestamp {
            found: error.to_string(),
        }
    }
}

impl From<SerdeJsonError> for TailscaleWebhookError {
    fn from(error: SerdeJsonError) -> Self {
        Self::Serde {
            error: error.to_string(),
        }
    }
}

impl From<io::Error> for TailscaleWebhookError {
    fn from(error: io::Error) -> Self {
        Self::IoError {
            error: error.to_string(),
        }
    }
}
