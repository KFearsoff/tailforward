#![allow(clippy::unwrap_used)]

use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;

use crate::tailscale::Event;

#[tracing::instrument]
pub async fn webhook_handler(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    let header_result = match headers.get("Tailscale-Webhook-Signature") {
        Some(val) => val.to_str().map_err(|_| StatusCode::UNPROCESSABLE_ENTITY),
        None => return StatusCode::UNPROCESSABLE_ENTITY,
    };

    let header = match header_result {
        Ok(str) => str,
        Err(code) => return code,
    };

    let Ok(payload) = serde_json::from_slice(&body) else { return StatusCode::UNPROCESSABLE_ENTITY };

    match Event::verify_webhook_sig(payload, header, body).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
