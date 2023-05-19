use crate::models::{error::TailscaleWebhook, event::Event};
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use color_eyre::Report;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::info;

#[tracing::instrument]
pub fn post_webhook(
    header: &str,
    body: &str,
    datetime: DateTime<Utc>,
    secret: &str,
) -> Result<Vec<Event>, Report> {
    // Axum extracts body as String with backslashes to escape double quotes.
    // The body is signed without those backslashes, so we trim them if they exist.
    // TODO: add tests
    let header = &header.replace('\\', "");
    let (t, v) = parse_header(header)?;
    let _timestamp = compare_timestamp(t, datetime)?;
    let string_to_sign = format!("{t}.{body}");
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
            info!(time_diff = other, "calculated time difference");
            timestamp
        }),
    }
}

#[tracing::instrument]
fn verify_sig(sig: &str, content: &str, secret: &str) -> Result<(), TailscaleWebhook> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())?;
    mac.update(content.as_bytes());
    let code_bytes = hex::decode(sig)?;
    mac.verify_slice(&code_bytes[..])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::{ExposeSecret, SecretString};
    use std::str::FromStr;

    #[test]
    fn post_webhook_good() {
        let body = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"test","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "42c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let timestamp_input: i64 = 1684518293;
        let datetime = chrono::Utc.timestamp_opt(timestamp_input, 0).unwrap();
        let secret = SecretString::from_str("123").unwrap();
        let header = format!("t={},v1={}", timestamp_input, v1_val);

        let out = post_webhook(&header, body, datetime, secret.expose_secret());
        out.unwrap();

        //assert!(out.is_ok());
    }

    #[test]
    fn header_good() {
        let header = "t=foo,v1=bar";
        let (t, v1) = parse_header(header).unwrap();
        assert_eq!(t, "foo");
        assert_eq!(v1, "bar");
    }

    #[test]
    fn header_no_comma() {
        let header = "t=foov1=bar";
        let out = parse_header(header);
        assert!(out.is_err());
    }

    #[test]
    fn header_more_commas() {
        let header = "t=foo,,v1=bar";
        let out = parse_header(header);
        assert!(out.is_err());
    }

    #[test]
    fn header_t_malformed() {
        let header = "tfoo,v1=bar";
        let out = parse_header(header);
        assert!(out.is_err());
    }

    #[test]
    fn header_v1_malformed() {
        let header = "t=foo,v1bar";
        let out = parse_header(header);
        assert!(out.is_err());
    }

    #[test]
    fn header_not_t() {
        let header = "a=foo,v1=bar";
        let out = parse_header(header);
        assert!(out.is_err());
    }

    #[test]
    fn header_not_v1() {
        let header = "t=foo,v=bar";
        let out = parse_header(header);
        assert!(out.is_err());
    }

    #[test]
    fn timestamp_good() {
        let timestamp_input: i64 = 1684518293;
        let timestamp = timestamp_input.to_string();
        let now = chrono::Utc.timestamp_opt(timestamp_input, 0).unwrap();
        let out = compare_timestamp(&timestamp, now);

        assert!(out.is_ok());
    }

    #[test]
    fn timestamp_newer() {
        let timestamp_input: i64 = 1684518293;
        let timestamp = timestamp_input.to_string();
        let now = chrono::Utc.timestamp_opt(timestamp_input - 1, 0).unwrap();
        let out = compare_timestamp(&timestamp, now);

        assert!(out.is_err());
    }

    #[test]
    fn timestamp_older_good() {
        let timestamp_input: i64 = 1684518293;
        let timestamp = timestamp_input.to_string();
        let now = chrono::Utc.timestamp_opt(timestamp_input + 299, 0).unwrap();
        let out = compare_timestamp(&timestamp, now);

        assert!(out.is_ok());
    }

    #[test]
    fn timestamp_older_equal() {
        let timestamp_input: i64 = 1684518293;
        let timestamp = timestamp_input.to_string();
        let now = chrono::Utc.timestamp_opt(timestamp_input + 300, 0).unwrap();
        let out = compare_timestamp(&timestamp, now);

        assert!(out.is_ok());
    }

    #[test]
    fn timestamp_older_newer() {
        let timestamp_input: i64 = 1684518293;
        let timestamp = timestamp_input.to_string();
        let now = chrono::Utc.timestamp_opt(timestamp_input + 301, 0).unwrap();
        let out = compare_timestamp(&timestamp, now);

        assert!(out.is_err());
    }

    #[test]
    fn sig_verify_good() {
        let json_str = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"test","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "42c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let secret = SecretString::from_str("123").unwrap();
        let input = format!("{}.{}", "1684518293", json_str);
        let out = verify_sig(v1_val, &input, secret.expose_secret());

        assert!(out.is_ok());
    }

    #[test]
    fn sig_verify_wrong_secret() {
        let json_str = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"test","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "42c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let secret = SecretString::from_str("1234").unwrap();
        let input = format!("{}.{}", "1684518293", json_str);
        let out = verify_sig(v1_val, &input, secret.expose_secret());

        assert!(out.is_err());
    }

    #[test]
    fn sig_verify_wrong_input() {
        let json_str = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"TEST","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "42c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let secret = SecretString::from_str("123").unwrap();
        let input = format!("{}.{}", "1684518293", json_str);
        let out = verify_sig(v1_val, &input, secret.expose_secret());

        assert!(out.is_err());
    }

    #[test]
    fn sig_verify_wrong_sig() {
        let json_str = r#"[{"timestamp":"2023-05-17T11:13:07.62352885Z","version":1,"type":"test","tailnet":"kfearsoff@gmail.com","message":"This is a test event"}]"#;
        // Generated with key "123" on https://www.freeformatter.com/hmac-generator.html
        let v1_val = "52c43ae89c3bbdc8e9c3a64ec9c2bf489159ef59a000aacaf9b880c5b617c9bb";
        let secret = SecretString::from_str("123").unwrap();
        let input = format!("{}.{}", "1684518293", json_str);
        let out = verify_sig(v1_val, &input, secret.expose_secret());

        assert!(out.is_err());
    }
}
