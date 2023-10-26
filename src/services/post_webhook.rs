use crate::models::{event::Event, Header};
use color_eyre::Report;
use hmac::{
    digest::{core_api::CoreWrapper, MacError},
    Hmac, HmacCore, Mac,
};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use tap::Tap;
use tracing::debug;

#[tracing::instrument]
pub fn post_webhook(
    header: Header,
    body: &str,
    secret: &SecretString,
) -> Result<Vec<Event>, Report> {
    let sig = hex::decode(header.signature.value)?;

    let string_to_sign = format!("{0}.{body}", header.timestamp.timestamp())
        .tap(|string| debug!(string, "Got string to sign"));
    let secret_exposed = secret
        .expose_secret()
        .tap_deref_dbg(|secret_value| debug!(secret_value));

    let mac = Hmac::<Sha256>::new_from_slice(secret_exposed.as_bytes())?;
    verify_sig(sig, &string_to_sign, mac)?;

    Ok(serde_json::from_str::<Vec<Event>>(body)?)
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
    use crate::models::tailscale_header::{Signature, Version};
    use chrono::{DateTime, Utc};
    use hmac::{digest::core_api::CoreWrapper, HmacCore};
    use secrecy::SecretString;
    use std::str::FromStr;
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
        mac.update(input.as_bytes());
        let v1_val = hex::encode(mac.finalize().into_bytes());
        let header = Header {
            timestamp,
            signature: Signature {
                version: Version::V1,
                value: v1_val,
            },
        };

        let out = post_webhook(header, &body_str, &secret);
        assert!(out.is_ok());
    }

    fn arrange_basics(secret: &str) -> (DateTime<Utc>, CoreWrapper<HmacCore<Sha256>>) {
        let timestamp = Utc::now();
        let hmac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        (timestamp, hmac)
    }

    fn arrange_sig_to_check(secret: &str, input: &str) -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(input.as_bytes());
        mac.finalize().into_bytes().to_vec()
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
