use super::TailscaleWebhook;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use std::str::FromStr;
use tracing::info;

#[derive(Debug)]
pub struct Header {
    pub timestamp: DateTime<Utc>,
    pub signature: Signature,
}

#[derive(Debug)]
pub struct Signature {
    pub version: Version,
    pub value: String,
}

#[derive(Debug)]
pub enum Version {
    V1,
    Unsupported,
}

impl FromStr for Signature {
    type Err = TailscaleWebhook;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let version = Version::V1;
        let hash = get_header_field(s, "v1", "v1=<signature>")?;
        Ok(Self {
            version,
            value: hash.to_owned(),
        })
    }
}

impl FromStr for Header {
    type Err = TailscaleWebhook;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let now = Utc::now();
        println!("{s:?}");
        let (t, v) = parse_header(s)?;
        println!("{t:?}");
        let timestamp_input: i64 = t.parse().map_err(TailscaleWebhook::from)?;
        println!("{timestamp_input}");
        let timestamp_verified: DateTime<Utc> = match chrono::Utc.timestamp_opt(timestamp_input, 0) {
            LocalResult::None => Err(TailscaleWebhook::TimestampDifference { found: timestamp_input }),
            LocalResult::Single(t) => Ok(t),
            LocalResult::Ambiguous(_, _) => unreachable!("A timestamp was parsed ambigiously. This should never happen with `timestamp_opt` function, so something has gone terribly wrong.")
        }?;
        println!("{0:?}", timestamp_verified.timestamp());
        let timestamp = match now.signed_duration_since(timestamp_verified).num_seconds() {
            x if x > 300 => Err(TailscaleWebhook::TimestampDifference { found: x }),
            x if x < 0 => Err(TailscaleWebhook::TimestampDifference { found: x }),
            other => Ok({
                info!(time_diff = other, "Calculated time difference");
                timestamp_verified
            }),
        }?;
        println!("{0:?}", timestamp.timestamp());
        Ok(Self {
            timestamp,
            signature: Signature {
                version: Version::V1,
                value: v.to_owned(),
            },
        })
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

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
    #[test_case(2 => matches Err(_); "when newer")]
    #[test_case(-299 => matches Ok(_); "when old lt")]
    #[test_case(-300 => matches Ok(_); "when old eq")]
    #[test_case(-301 => matches Err(_); "when old gt")]
    fn timestamp_correct(correction: i64) -> Result<DateTime<Utc>, TailscaleWebhook> {
        let timestamp = Utc::now().timestamp();
        let now = chrono::Utc
            .timestamp_opt(timestamp + correction, 0)
            .unwrap()
            .timestamp();
        let header = Header::from_str(&format!("t={0},v1=ss", now))?;
        println!("{0}", &header.timestamp.timestamp());
        Ok(header.timestamp)
    }
}
