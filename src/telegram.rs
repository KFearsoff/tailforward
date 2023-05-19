use crate::models::event::Event;
use reqwest::{Result, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize)]
pub struct Message {
    chat_id: i64,
    text: String,
}

const CHAT_ID: i64 = -1_001_864_190_705;

#[warn(clippy::unwrap_used)]
impl Message {
    pub async fn post(events: Vec<Event>) -> Result<StatusCode> {
        let client = reqwest::Client::new();
        let secret: SecretString = fs::read_to_string("/secrets/telegram")
            .await
            .unwrap()
            .split('=')
            .collect::<Vec<_>>()[1]
            .to_string()
            .into();

        let text = events.iter().map(|event| Self {
            chat_id: CHAT_ID,
            text: format!("{event:?}"),
        });

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            secret.expose_secret()
        );
        for message in text {
            client.post(&url).json(&message).send().await?;
        }
        Ok(StatusCode::OK)
    }
}
