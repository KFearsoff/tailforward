use hex::FromHexError;
use hmac::digest::{InvalidLength, MacError};
use serde_json::Error;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TailscaleWebhook {
    #[error("webhook has an invalid signature")]
    InvalidSignature,
    #[error("webhook has an invalid header (expected: {expected}, got: {got})")]
    InvalidHeader { expected: String, got: String },
    #[error("Tailscale-Webhook-Signature header is empty")]
    EmptyHeader,
    #[error("the difference in timestamp is too large ({found}s)")]
    TimestampDifference { found: i64 },
    #[error("error parsing int: {source}")]
    ParseIntError {
        #[from]
        source: ParseIntError,
    },
    #[error("error decoding mac")]
    MacError {
        #[from]
        source: MacError,
    },
    #[error("error passing in secret")]
    InvalidLength {
        #[from]
        source: InvalidLength,
    },
    #[error("error decoding hex")]
    FromHexError {
        #[from]
        source: FromHexError,
    },
    #[error("error unmarshaling json")]
    SerdeJson {
        #[from]
        source: Error,
    },
}
