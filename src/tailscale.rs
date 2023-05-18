use crate::errors::TailscaleWebhookError;
use axum::body::Bytes;
use bytes::{BufMut, BytesMut};
use chrono::{DateTime, Utc};
use chrono::{LocalResult, TimeZone};
use derive_more::Display;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use sha2::Sha256;
use tokio::fs;
use tracing::{info, warn};

#[derive(Serialize, Deserialize, Debug, Display)]
#[display(fmt = "Event data: {timestamp}, {version}, {type}, {tailnet}, {message}")]
pub struct Event {
    timestamp: String,
    version: i8,
    r#type: String,
    tailnet: String,
    message: String,
    #[display("{data:#?}")]
    data: Option<Value>,
}

impl Event {
    pub async fn get(header: &str, body: String) -> Result<Vec<Self>, TailscaleWebhookError> {
        let secret: SecretString = fs::read_to_string("/secrets/tailscale-webhook")
            .await
            .map_err(TailscaleWebhookError::from)?
            .into();
        let verified = Self::verify_webhook_sig(secret, header, &body);
        match verified {
            Ok(_) => serde_json::from_str(&body).map_err(TailscaleWebhookError::from),
            Err(_) => Err(TailscaleWebhookError::NotSigned),
        }
    }

    #[warn(clippy::expect_used)]
    #[tracing::instrument]
    pub fn verify_webhook_sig(
        secret: SecretString,
        header: &str,
        body: &str,
    ) -> Result<(), TailscaleWebhookError> {
        let (t_value, v1_value) = Self::parse_sig_header(header)?;
        let (stamp, hash) = Self::validate_values(t_value, &v1_value)?;
        let unix_timestamp = stamp.timestamp();

        info!(unix_timestamp);
        let timestamp_bytes = unix_timestamp.to_be_bytes();

        let json_str = body;

        //        let mut buf = BytesMut::new();
        //        buf.put_slice(&timestamp_bytes);
        //        buf.put_slice(b".");
        //        buf.put_slice(body);
        //
        //        let mut mac = Hmac::<Sha256>::new_from_slice(secret.expose_secret().as_bytes())
        //            .expect("HMAC can take key of any size");
        //        mac.update(&buf);
        //
        //        mac.verify_slice(hash)
        //            .map_err(|_| TailscaleWebhookError::NotSigned)

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.expose_secret().as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(json_str.as_bytes());

        let code_bytes = hex::decode(v1_value).expect("Can't decode hex");
        mac.verify_slice(&code_bytes[..])
            .map_err(|_| TailscaleWebhookError::NotSigned)
    }

    #[tracing::instrument]
    fn parse_sig_header(header: &str) -> Result<(String, String), TailscaleWebhookError> {
        if header.is_empty() {
            return Err(TailscaleWebhookError::InvalidHeader {
                error: "empty header".to_string(),
            });
        };

        let parts: Vec<&str> = header.split(',').collect();
        if parts.len() != 2 {
            return Err(TailscaleWebhookError::InvalidHeader {
                error: format!(
                    "expected t=<unix timestamp>,v1=<signature>, got {}",
                    parts.join(",")
                ),
            });
        }

        let t_part = parts[0];
        let v1_part = parts[1];

        let t_value =
            t_part
                .strip_prefix("t=")
                .ok_or_else(|| TailscaleWebhookError::InvalidHeader {
                    error: format!("expected t=<unix timestamp>, got {t_part}"),
                })?;

        let v1_value =
            v1_part
                .strip_prefix("v1=")
                .ok_or_else(|| TailscaleWebhookError::InvalidHeader {
                    error: format!("expected v1=<signature>, got {v1_part}"),
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

        let now = Utc::now();
        match now.signed_duration_since(timestamp).num_seconds() {
            x if x > 300 => Err(TailscaleWebhookError::IncorrectTimestamp {
                found: "too old".to_string(),
            }),
            x if x < 0 => Err(TailscaleWebhookError::IncorrectTimestamp {
                found: "negative timestamp".to_string(),
            }),
            other => Ok({
                info!(time_diff = other, "calculated time difference");
                timestamp
            }),
        }
    }

    #[tracing::instrument]
    fn validate_v1(v1: &str) -> Result<&[u8], TailscaleWebhookError> {
        Ok(v1.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn empty_header() {
        let result = Event::parse_sig_header("").unwrap_err();
        assert_eq!(
            result,
            TailscaleWebhookError::InvalidHeader {
                error: "empty header".to_string()
            }
        );
    }

    #[test]
    fn wrong_args_header() {
        let input = "foo,bar,baz";
        let result = Event::parse_sig_header(input).unwrap_err();
        assert_eq!(
            result,
            TailscaleWebhookError::InvalidHeader {
                error: format!("expected t=<unix timestamp>,v1=<signature>, got {input}")
            }
        );
    }

    #[test]
    fn invalid_header() {
        let input = "foo,v1=bar";
        let result = Event::parse_sig_header(input).unwrap_err();
        assert_eq!(
            result,
            TailscaleWebhookError::InvalidHeader {
                error: format!(
                    "expected t=<unix timestamp>, got {}",
                    input.split(',').collect::<Vec<&str>>()[0]
                )
            }
        );

        let input = "t=123,bar";
        let result = Event::parse_sig_header(input).unwrap_err();
        assert_eq!(
            result,
            TailscaleWebhookError::InvalidHeader {
                error: format!(
                    "expected v1=<signature>, got {}",
                    input.split(',').collect::<Vec<&str>>()[1]
                )
            }
        );
    }

    #[test]
    fn correct_header() {
        let result = Event::parse_sig_header("t=foo,v1=bar").unwrap();
        assert_eq!(result, ("foo".to_string(), "bar".to_string()));
    }

    #[test]
    fn correct_validation() {
        let json_str = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"test","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "a738edd5854dbdea2e94692bf5791309a6b07efa537c2f125a6bddb2a0c18151";
        let secret = SecretString::from_str("123").unwrap();
        let input = format!("{}.{}", "1684356735", json_str);

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.expose_secret().as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(input.as_bytes());

        let code_bytes = hex::decode(v1_val).unwrap();
        mac.verify_slice(&code_bytes[..]).unwrap();
    }

    // #[test]
    // fn invalid_version() {
    //     let result = Event::parse_sig_header("t=foo,v1=bar").unwrap_err();
    //     assert_eq!(result, TailscaleWebhookError::IncorrectTimestamp {
    //             found: "invalid digit found in string".to_string(),
    //     });
    // }
}
