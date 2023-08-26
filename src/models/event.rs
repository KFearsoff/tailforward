use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub version: u8,
    pub r#type: String,
    pub tailnet: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
