#![allow(clippy::unwrap_used)]

use axum::response::IntoResponse;
use serde_json::{json, Value};
use tracing::info;

use crate::tailscale::Event;

#[tracing::instrument]
pub async fn webhook_handler() -> impl IntoResponse {
    Event::verify_webhook_sig().await.unwrap()
}
