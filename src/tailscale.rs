use crate::{errors::TailscaleWebhookError, SECRET};
use axum::body::Bytes;
use bytes::{BufMut, BytesMut};
use chrono::{DateTime, Utc};
use chrono::{LocalResult, TimeZone};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tracing::{info, warn};

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    timestamp: String,
    version: i8,
    r#type: String,
    tailnet: String,
    message: String,
    data: String,
}

impl Event {
    #[warn(clippy::unused_async, clippy::expect_used)]
    #[tracing::instrument]
    pub async fn verify_webhook_sig(
        self,
        header: &str,
        body: Bytes,
    ) -> Result<Self, TailscaleWebhookError> {
        let (t_value, v1_value) = Self::parse_sig_header(header)?;
        let (stamp, hash) = Self::validate_values(t_value, &v1_value)?;
        let unix_timestamp = stamp.timestamp();

        info!(unix_timestamp);
        let timestamp_bytes = unix_timestamp.to_be_bytes();

        let mut buf = BytesMut::new();
        buf.put_slice(&timestamp_bytes);
        buf.put_slice(b".");
        buf.put_slice(&body);

        let mut mac = Hmac::<Sha256>::new_from_slice(SECRET.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(&buf);

        match mac.verify_slice(hash) {
            Ok(_) => serde_json::from_slice(&body).map_err(TailscaleWebhookError::from),
            Err(_) => Err(TailscaleWebhookError::NotSigned),
        }
    }

    #[tracing::instrument]
    fn parse_sig_header(header: &str) -> Result<(String, String), TailscaleWebhookError> {
        if header.is_empty() {
            return Err(TailscaleWebhookError::EmptyHeader);
        };

        let parts: Vec<&str> = header.split(',').collect();
        if parts.len() != 2 {
            return Err(TailscaleWebhookError::InvalidHeader {
                expected: "t=<unix timestamp>,v1=<signature>".to_string(),
                found: parts.join(","),
            });
        }

        let t_part = parts[0];
        let v1_part = parts[1];

        let t_value =
            t_part
                .strip_prefix("t=")
                .ok_or_else(|| TailscaleWebhookError::InvalidHeader {
                    expected: "t=<unix timestamp>".to_string(),
                    found: t_part.to_string(),
                })?;

        let v1_value =
            v1_part
                .strip_prefix("v1=")
                .ok_or_else(|| TailscaleWebhookError::InvalidHeader {
                    expected: "v1=<signature>".to_string(),
                    found: v1_part.to_string(),
                })?;

        Ok((t_value.to_string(), v1_value.to_string()))
    }

    #[tracing::instrument]
    fn validate_values(
        t: String,
        v1: &str,
    ) -> Result<(DateTime<Utc>, &[u8]), TailscaleWebhookError> {
        let timestamp = Self::validate_t(t)?;
        let hash = Self::validate_v1(v1)?;
        Ok((timestamp, hash))
    }

    #[tracing::instrument]
    fn validate_t(t: String) -> Result<DateTime<Utc>, TailscaleWebhookError> {
        let t_value = t.parse::<i64>().map_err(TailscaleWebhookError::from)?;
        let timestamp: DateTime<Utc> = match chrono::Utc.timestamp_opt(t_value, 0) {
            LocalResult::None => Err(TailscaleWebhookError::IncorrectTimestamp { found: t_value.to_string() }),
            LocalResult::Single(t) => Ok(t),
            LocalResult::Ambiguous(_, _) => unreachable!("A timestamp was parsed ambigiously. This should never happen with `timestamp_opt` function, so something has gone terribly wrong.")
        }?;
        Ok(timestamp)
    }

    #[tracing::instrument]
    fn validate_v1(v1: &str) -> Result<&[u8], TailscaleWebhookError> {
        Ok(v1.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_header() {
        let result = Event::parse_sig_header("").unwrap_err();
        assert_eq!(result, TailscaleWebhookError::EmptyHeader);
    }

    #[test]
    fn wrong_args_header() {
        let result = Event::parse_sig_header("foo,bar,baz").unwrap_err();
        assert_eq!(
            result,
            TailscaleWebhookError::InvalidHeader {
                expected: "t=<unix timestamp>,v1=<signature>".to_string(),
                found: "foo,bar,baz".to_string(),
            }
        );
    }

    #[test]
    fn invalid_header() {
        let result = Event::parse_sig_header("foo,v1=bar").unwrap_err();
        assert_eq!(
            result,
            TailscaleWebhookError::InvalidHeader {
                expected: "t=<unix timestamp>".to_string(),
                found: "foo".to_string(),
            }
        );

        let result = Event::parse_sig_header("t=123,bar").unwrap_err();
        assert_eq!(
            result,
            TailscaleWebhookError::InvalidHeader {
                expected: "v1=<signature>".to_string(),
                found: "bar".to_string(),
            }
        );
    }

    #[test]
    fn correct_header() {
        let result = Event::parse_sig_header("t=foo,v1=bar").unwrap();
        assert_eq!(result, ("foo".to_string(), "bar".to_string()));
    }

    // #[test]
    // fn invalid_version() {
    //     let result = Event::parse_sig_header("t=foo,v1=bar").unwrap_err();
    //     assert_eq!(result, TailscaleWebhookError::IncorrectTimestamp {
    //             found: "invalid digit found in string".to_string(),
    //     });
    // }
}
