use crate::models::{event::Event, Header};
use color_eyre::Report;
use hmac::{Hmac, Mac};
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

    let mut mac = Hmac::<Sha256>::new_from_slice(secret_exposed.as_bytes())?;
    mac.update(string_to_sign.as_bytes());
    mac.verify_slice(&sig)?;

    Ok(serde_json::from_str::<Vec<Event>>(body)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tailscale_header::{Signature, Version};
    use chrono::Utc;
    use secrecy::SecretString;
    use std::str::FromStr;
    use test_case::test_case;

    #[test_case("123" => matches Ok(_); "when correct")]
    #[test_case("1234" => matches Err(_); "when incorrect")]
    fn is_webhook_good(secret_act: &str) -> Result<Vec<Event>, Report> {
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

        let mut mac = Hmac::<Sha256>::new_from_slice(secret_act.as_bytes()).unwrap();
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

        post_webhook(header, &body_str, &secret)
    }
}
