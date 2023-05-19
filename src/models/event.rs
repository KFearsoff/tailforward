use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    timestamp: DateTime<Utc>,
    version: u8,
    r#type: String,
    tailnet: String,
    message: String,
    data: Option<Value>,
}
