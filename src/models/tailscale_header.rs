use super::TailscaleWebhook;
use chrono::{DateTime, Utc};
use derive_more::Display;
use std::str::FromStr;
use tracing::info;

#[derive(Debug)]
pub struct Header {
    pub timestamp: DateTime<Utc>,
    pub signature: Signature,
}

impl FromStr for Header {
    type Err = TailscaleWebhook;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let now = Utc::now();
        let (timestamp_parsed, signature) = parse_header(s)?;

        let timestamp = match now.signed_duration_since(timestamp_parsed).num_seconds() {
            x if x > 300 => Err(TailscaleWebhook::TimestampDifference { found: x }),
            x if x < 0 => Err(TailscaleWebhook::TimestampDifference { found: x }),
            other => Ok({
                info!(time_diff = other, "Calculated time difference");
                timestamp_parsed
            }),
        }?;

        Ok(Self {
            timestamp,
            signature,
        })
    }
}

#[tracing::instrument]
fn parse_header(header: &str) -> Result<(DateTime<Utc>, Signature), TailscaleWebhook> {
    let (t, v): (&str, &str) =
        header
            .split_once(',')
            .ok_or_else(|| TailscaleWebhook::InvalidHeader {
                expected: "t=<timestamp>,v1=<signature>".to_string(),
                got: header.to_string(),
            })?;
    let timestamp = get_timestamp(t)?;
    let hash = Signature::from_str(v)?;
    Ok((timestamp, hash))
}

#[tracing::instrument]
fn get_timestamp(field: &str) -> Result<DateTime<Utc>, TailscaleWebhook> {
    let (key, timestamp) =
        field
            .split_once('=')
            .ok_or_else(|| TailscaleWebhook::InvalidHeader {
                expected: "t=<timestamp>".to_string(),
                got: field.to_string(),
            })?;
    if key == "t" {
        let parsed = timestamp.parse()?;
        DateTime::from_timestamp(parsed, 0).ok_or(TailscaleWebhook::InvalidHeader {
            expected: "t".to_owned(),
            got: key.to_owned(),
        })
    } else {
        Err(TailscaleWebhook::InvalidHeader {
            expected: "t".to_string(),
            got: key.to_string(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Signature {
    pub version: Version,
    pub value: String,
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        self.value.as_ref()
    }
}

impl FromStr for Signature {
    type Err = TailscaleWebhook;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (version, hash) = s
            .split_once('=')
            .ok_or_else(|| TailscaleWebhook::InvalidHeader {
                expected: "v1=<signature>".to_string(),
                got: s.to_string(),
            })?;
        let version = Version::from_str(version)?;
        Ok(Self {
            version,
            value: hash.to_owned(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Display)]
pub enum Version {
    #[display(fmt = "v1")]
    V1,
}

impl FromStr for Version {
    type Err = TailscaleWebhook;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1" => Ok(Self::V1),
            _ => Err(TailscaleWebhook::EmptyHeader),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use test_case::test_case;

    #[test_case("t=123,v1=bar" => matches Ok(_); "when correct")]
    #[test_case("t=123f,v1=bar" => matches Err(_); "when timestamp invalid")]
    #[test_case("t=123v1=bar" => matches Err(_); "when no comma")]
    #[test_case("t=123,,v1=bar" => matches Err(_); "when too many commas")]
    #[test_case("t123,v1=bar" => matches Err(_); "when t is malformed")]
    #[test_case("t=123,v1bar" => matches Err(_); "when v1 is malformed")]
    #[test_case("a=123,v1=bar" => matches Err(_); "when header is not t")]
    #[test_case("t=123,v=bar" => matches Err(_); "when header is not v1")]
    #[test_case("t=123,v2=bar" => matches Err(_); "when header is v!=1")]
    fn is_header_correct(header: &str) -> Result<(DateTime<Utc>, Signature), TailscaleWebhook> {
        parse_header(header)
    }

    #[test_case(0 => matches Ok(_); "when equal")]
    #[test_case(2 => matches Err(_); "when newer")]
    #[test_case(-299 => matches Ok(_); "when old lt")]
    #[test_case(-300 => matches Ok(_); "when old eq")]
    #[test_case(-301 => matches Err(_); "when old gt")]
    fn timestamp_correct(correction: i64) -> Result<Header, TailscaleWebhook> {
        let timestamp = Utc::now().timestamp();
        let now = chrono::Utc
            .timestamp_opt(timestamp + correction, 0)
            .unwrap()
            .timestamp();
        println!("{now:?}");
        let header = Header::from_str(&format!("t={0},v1=ss", now))?;
        Ok(header)
    }
}
