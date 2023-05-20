use crate::models::{event::Event, message::Message};
use crate::CHAT_ID;
use color_eyre::Report;
use secrecy::{ExposeSecret, SecretString};
use tracing::{debug, info};

/// # Errors
///
/// `reqwest::Error`
pub async fn post(
    events: Vec<Event>,
    client: reqwest::Client,
    secret: SecretString,
) -> Result<(), Report> {
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
