use crate::models::{error::TailscaleWebhook, event::Event};
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use color_eyre::Report;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use tap::Tap;
use tracing::{debug, info};

#[tracing::instrument]
pub fn post_webhook(
    header: &str,
    body: &str,
    datetime: DateTime<Utc>,
    secret: &SecretString,
) -> Result<Vec<Event>, Report> {
    let (t, v) = parse_header(header)?;
    let _timestamp = compare_timestamp(t, datetime)?;

    let string_to_sign = format!("{t}.{body}").tap(|string| debug!(string, "Got string to sign"));
    verify_sig(v, &string_to_sign, secret)?;

    Ok(serde_json::from_str::<Vec<Event>>(body)?)
}

#[tracing::instrument]
fn parse_header(header: &str) -> Result<(&str, &str), TailscaleWebhook> {
    let (t, v): (&str, &str) =
        header
            .split_once(',')
            .ok_or_else(|| TailscaleWebhook::InvalidHeader {
                expected: "t=<timestamp>,v1=<signature>".to_string(),
                got: header.to_string(),
            })?;
    let timestamp = get_header_field(t, "t", "t=<timestamp>")?;
    let hash = get_header_field(v, "v1", "v1=<signature>")?;
    Ok((timestamp, hash))
}

#[tracing::instrument]
fn get_header_field<'a>(
    field: &'a str,
    name: &str,
    err_message: &str,
) -> Result<&'a str, TailscaleWebhook> {
    let (first, second) = field
        .split_once('=')
        .ok_or_else(|| TailscaleWebhook::InvalidHeader {
            expected: err_message.to_string(),
            got: field.to_string(),
        })?;
    if first == name {
        Ok(second)
    } else {
        Err(TailscaleWebhook::InvalidHeader {
            expected: name.to_string(),
            got: first.to_string(),
        })
    }
}

#[tracing::instrument]
fn compare_timestamp(
    parsed: &str,
    current: DateTime<Utc>,
) -> Result<DateTime<Utc>, TailscaleWebhook> {
    let timestamp: i64 = parsed.parse().map_err(TailscaleWebhook::from)?;
    let timestamp: DateTime<Utc> = match chrono::Utc.timestamp_opt(timestamp, 0) {
        LocalResult::None => Err(TailscaleWebhook::TimestampDifference { found: timestamp }),
        LocalResult::Single(t) => Ok(t),
        LocalResult::Ambiguous(_, _) => unreachable!("A timestamp was parsed ambigiously. This should never happen with `timestamp_opt` function, so something has gone terribly wrong.")
    }?;

    match current.signed_duration_since(timestamp).num_seconds() {
        x if x > 300 => Err(TailscaleWebhook::TimestampDifference { found: x }),
        x if x < 0 => Err(TailscaleWebhook::TimestampDifference { found: x }),
        other => Ok({
            info!(time_diff = other, "Calculated time difference");
            timestamp
        }),
    }
}

#[tracing::instrument]
#[allow(clippy::unwrap_used)]
fn verify_sig(sig: &str, content: &str, secret: &SecretString) -> Result<(), TailscaleWebhook> {
    // Axum extracts body as String with backslashes to escape double quotes.
    // The body is signed without those backslashes, so we trim them if they exist.
    debug!(input_length = content.len());
    let stripped: &str = &content
        .replace('\\', "")
        .tap(|str| debug!(stripped_length = str.len()));
    let secret_exposed = secret
        .expose_secret()
        .tap_deref_dbg(|secret_value| debug!(secret_value));

    let mut mac = Hmac::<Sha256>::new_from_slice(secret_exposed.as_bytes())?;
    mac.update(stripped.as_bytes());
    let code_bytes = hex::decode(sig)?;
    mac.verify_slice(&code_bytes[..])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use secrecy::SecretString;
    use std::str::FromStr;
    use test_case::test_case;

    #[test]
    fn post_webhook_good() {
        let body = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"test","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "42c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let timestamp_input: i64 = 1684518293;
        let datetime = chrono::Utc.timestamp_opt(timestamp_input, 0).unwrap();
        let secret = SecretString::from_str("123").unwrap();
        let header = format!("t={},v1={}", timestamp_input, v1_val);

        let out = post_webhook(&header, body, datetime, &secret);
        assert!(out.is_ok());
    }

    #[test_case("t=foo,v1=bar" => matches Ok(_); "when correct")]
    #[test_case("t=foov1=bar" => matches Err(_); "when no comma")]
    #[test_case("t=foo,,v1=bar" => matches Err(_); "when too many commas")]
    #[test_case("tfoo,v1=bar" => matches Err(_); "when t is malformed")]
    #[test_case("t=foo,v1bar" => matches Err(_); "when v1 is malformed")]
    #[test_case("a=foo,v1=bar" => matches Err(_); "when header is not t")]
    #[test_case("t=foo,v=bar" => matches Err(_); "when header is not v1")]
    #[test_case("t=foo,v2=bar" => matches Err(_); "when header is v!=1")]
    fn is_header_correct(header: &str) -> Result<(&str, &str), TailscaleWebhook> {
        let out = parse_header(header);
        out
    }

    #[test_case(0 => matches Ok(_); "when equal")]
    #[test_case(-1 => matches Err(_); "when newer")]
    #[test_case(299 => matches Ok(_); "when old lt")]
    #[test_case(300 => matches Ok(_); "when old eq")]
    #[test_case(301 => matches Err(_); "when old gt")]
    fn timestamp_correct(correction: i64) -> Result<DateTime<Utc>, TailscaleWebhook> {
        let timestamp_input: i64 = 1684518293;
        let timestamp = timestamp_input.to_string();
        let now = chrono::Utc
            .timestamp_opt(timestamp_input + correction, 0)
            .unwrap();
        let out = compare_timestamp(&timestamp, now);
        out
    }

    #[test]
    fn sig_verify_good() {
        let json_str = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"test","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "42c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let secret = SecretString::from_str("123").unwrap();
        let input = format!("{}.{}", "1684518293", json_str);
        let out = verify_sig(v1_val, &input, &secret);

        assert!(out.is_ok());
    }

    #[test]
    fn sig_verify_backslashes_good() {
        let json_str = r#"[{\"timestamp\":\"2023-05-17T11:13:07.62352885Z\",\"version\":1,\"type\":\"test\",\"tailnet\":\"kfearsoff@gmail.com\",\"message\":\"This is a test event\"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "42c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let secret = SecretString::from_str("123").unwrap();
        let input = format!("{}.{}", "1684518293", json_str);
        let out = verify_sig(v1_val, &input, &secret);

        assert!(out.is_ok());
    }

    #[test]
    fn trim_backslashes() {
        let input = r#"[{\"timestamp\":\"2023-05-19T17:15:05.137256149Z\",\"version\":1,\"type\":\"test\",\"tailnet\":\"kfearsoff@gmail.com\",\"message\":\"This is a test event\"}]"#;
        let output = input.replace('\\', "");
        let condition = output.contains('\\');

        assert!(!condition);
    }

    #[test]
    fn trim_backslashes_len() {
        let input = r#"[{\"timestamp\":\"2023-05-19T17:15:05.137256149Z\",\"version\":1,\"type\":\"test\",\"tailnet\":\"kfearsoff@gmail.com\",\"message\":\"This is a test event\"}]"#;
        let input_len = input.len();
        let output = input.replace('\\', "");
        let output_len = output.len();

        assert_ne!(input_len, output_len);
    }

    #[test]
    fn sig_verify_wrong_secret() {
        let json_str = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"test","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "42c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let secret = SecretString::from_str("1234").unwrap();
        let input = format!("{}.{}", "1684518293", json_str);
        let out = verify_sig(v1_val, &input, &secret);

        assert!(out.is_err());
    }

    #[test]
    fn sig_verify_wrong_input() {
        let json_str = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"TEST","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "42c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let secret = SecretString::from_str("123").unwrap();
        let input = format!("{}.{}", "1684518293", json_str);
        let out = verify_sig(v1_val, &input, &secret);

        assert!(out.is_err());
    }

    #[test]
    fn sig_verify_wrong_sig() {
        let json_str = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"test","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "52c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let secret = SecretString::from_str("123").unwrap();
        let input = format!("{}.{}", "1684518293", json_str);
        let out = verify_sig(v1_val, &input, &secret);

        assert!(out.is_err());
    }
}
