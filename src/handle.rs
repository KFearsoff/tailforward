use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use tracing::{error, info};

use crate::tailscale::Event;

#[tracing::instrument]
pub async fn webhook_handler(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
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

    let payload = match serde_json::from_slice(&body) {
        Ok(val) => val,
        Err(err) => {
            return {
                error!(err = format!("{err}"), "Can't unmarshal JSON!");
                StatusCode::UNPROCESSABLE_ENTITY
            }
        }
    };

    match Event::verify_webhook_sig(payload, header, body).await {
        Ok(val) => {
            info!("{val}");
            StatusCode::OK
        }
        Err(err) => {
            error!("{err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
