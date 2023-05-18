use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use tracing::{error, info};

use crate::tailscale::Event;
use crate::telegram::Message;

#[tracing::instrument]
pub async fn webhook_handler(headers: HeaderMap, body: String) -> impl IntoResponse {
    let header_result = match headers.get("Tailscale-Webhook-Signature") {
        Some(val) => val.to_str().map_err(|err| {
            error!("{err}");
            StatusCode::UNPROCESSABLE_ENTITY
        }),
        None => {
            return {
                error!("No Tailscale-Webhook-Signature!");
                StatusCode::UNPROCESSABLE_ENTITY
            }
        }
    };

    let header = match header_result {
        Ok(str) => str,
        Err(code) => {
            return {
                error!("Failed to parse Tailscale-Webhook-Signature!");
                code
            }
        }
    };

    let events = match Event::get(header, body).await {
        Ok(val) => val,
        Err(err) => {
            return {
                error!("{err}");
                StatusCode::UNPROCESSABLE_ENTITY
            }
        }
    };
    info!(events = "{events:?}", "Got events");

    match Message::post(events).await {
        Ok(status) => status,
        Err(err) => {
            let nice_err = err.without_url();
            error!("{nice_err:?}");
            nice_err
                .status()
                .map_or(StatusCode::INTERNAL_SERVER_ERROR, |code| code)
        }
    }
}
