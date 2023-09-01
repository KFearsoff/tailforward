use crate::models::{error::TailscaleWebhook, event::Event};
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use color_eyre::Report;
use hmac::{
    digest::{core_api::CoreWrapper, MacError},
    Hmac, HmacCore, Mac,
};
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
    let sig = hex::decode(v)?;
    let _timestamp = compare_timestamp(t, datetime)?;

    let string_to_sign = format!("{t}.{body}").tap(|string| debug!(string, "Got string to sign"));
    let secret_exposed = secret
        .expose_secret()
        .tap_deref_dbg(|secret_value| debug!(secret_value));

    let mac = Hmac::<Sha256>::new_from_slice(secret_exposed.as_bytes())?;
    verify_sig(sig, &string_to_sign, mac)?;

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
fn verify_sig(
    sig_to_check: Vec<u8>,
    string_to_sign: &str,
    mut mac: CoreWrapper<HmacCore<Sha256>>,
) -> Result<(), MacError> {
    mac.update(string_to_sign.as_bytes());
    mac.verify_slice(&sig_to_check)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::{digest::core_api::CoreWrapper, HmacCore};
    use secrecy::SecretString;
    use std::str::FromStr;
    use test_case::test_case;
    use test_strategy::proptest;

    #[test]
    fn post_webhook_good() {
        let timestamp = Utc::now();
        let secret_str = "123";
        let secret = SecretString::from_str(secret_str).unwrap();

        let body_json = vec![Event {
            timestamp,
            version: 1,
            r#type: "test".to_owned(),
            tailnet: "example.com".to_owned(),
            message: "This is a test event".to_owned(),
            data: None,
        }];
        let body_str = serde_json::to_string(&body_json).unwrap();
        let mut mac = Hmac::<Sha256>::new_from_slice(secret_str.as_bytes()).unwrap();
        let input = format!("{}.{}", timestamp.timestamp(), body_str);
        mac.update(&input.as_bytes());
        let v1_val = hex::encode(mac.finalize().into_bytes());
        let header = format!("t={},v1={}", timestamp.timestamp(), v1_val);

        let out = post_webhook(&header, &body_str, timestamp, &secret);
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

    fn arrange_basics(secret: &str) -> (DateTime<Utc>, CoreWrapper<HmacCore<Sha256>>) {
        let timestamp = Utc::now();
        let hmac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        (timestamp, hmac)
    }

    fn arrange_sig_to_check(secret: &str, input: &str) -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(&input.as_bytes());
        let out = mac.finalize().into_bytes().to_vec();
        out
    }

    fn arrange_body(timestamp: DateTime<Utc>) -> Vec<Event> {
        vec![Event {
            timestamp,
            version: 1,
            r#type: "test".to_owned(),
            tailnet: "example.com".to_owned(),
            message: "This is a test event".to_owned(),
            data: None,
        }]
    }

    fn arrange_string_to_sign(timestamp: DateTime<Utc>, body: Vec<Event>) -> String {
        let body_str = serde_json::to_string(&body).unwrap();
        format!("{}.{}", timestamp.timestamp(), body_str)
    }

    #[test]
    fn verify_sig_passes() {
        let secret = "123";
        let (timestamp, mac) = arrange_basics(secret);

        let body_json = arrange_body(timestamp);
        let string_to_sign = arrange_string_to_sign(timestamp, body_json);
        let sig_to_check = arrange_sig_to_check(secret, &string_to_sign);

        let out = verify_sig(sig_to_check, &string_to_sign, mac);
        assert!(out.is_ok());
    }

    #[proptest]
    fn verify_sig_wrong_mac_fails(#[strategy(r"[^\x00]")] x: String) {
        let secret = "123";
        let (timestamp, _) = arrange_basics(secret);

        let body_json = arrange_body(timestamp);
        let string_to_sign = arrange_string_to_sign(timestamp, body_json);
        let sig_to_check = arrange_sig_to_check(secret, &string_to_sign);

        let (_, mac) = arrange_basics(&format!("{secret}{x}"));
        let out = verify_sig(sig_to_check, &string_to_sign, mac);
        assert!(out.is_err());
    }

    #[proptest]
    fn verify_sig_wrong_string_to_sign_fails(#[strategy(r".")] x: String) {
        let secret = "123";
        let (timestamp, hmac) = arrange_basics(secret);

        let body_json = arrange_body(timestamp);
        let string_to_sign = arrange_string_to_sign(timestamp, body_json);
        let sig_to_check = arrange_sig_to_check(secret, &string_to_sign);

        let string_to_sign = format!("{string_to_sign}{x}");
        let out = verify_sig(sig_to_check, &string_to_sign, hmac);
        assert!(out.is_err());
    }

    #[proptest]
    fn verify_sig_wrong_sig_to_check_fails(#[strategy(r".")] x: String) {
        let secret = "123";
        let (timestamp, mac) = arrange_basics(secret);

        let body_json = arrange_body(timestamp);
        let string_to_sign = arrange_string_to_sign(timestamp, body_json);
        let _sig_to_check = arrange_sig_to_check(secret, &string_to_sign);

        let input_broken = format!("{string_to_sign}{x}");
        let sig_to_check = arrange_sig_to_check(secret, &input_broken);
        let out = verify_sig(sig_to_check, &string_to_sign, mac);
        assert!(out.is_err());
    }
}
