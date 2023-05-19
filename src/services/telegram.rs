use crate::models::{event::Event, message::Message};
use crate::CHAT_ID;
use color_eyre::Report;
use secrecy::{ExposeSecret, SecretString};
use tokio::fs;
use tracing::{debug, info};

/// # Errors
///
/// `reqwest::Error`
pub async fn post(events: Vec<Event>) -> Result<(), Report> {
    let client = reqwest::Client::new();
    let secret: SecretString = fs::read_to_string("/secrets/telegram")
        .await?
        .split('=')
        .collect::<Vec<_>>()[1]
        .to_string()
        .into();
    info!("Read Telegram secret");

    let text = events.iter().map(|event| Message {
        chat_id: CHAT_ID,
        text: format!("{event:?}"),
    });
    info!("Mapped events to text");

    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        secret.expose_secret()
    );

    for message in text {
        debug!(message = "{message:?}", "Sending message");
        client.post(&url).json(&message).send().await?;
        info!(message = "{message:?}", "Sent message");
    }
    Ok(())
}
