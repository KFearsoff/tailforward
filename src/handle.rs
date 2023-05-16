#![allow(clippy::unwrap_used)]

use axum::extract::Json;
use axum::response::IntoResponse;

use crate::tailscale::Event;

#[tracing::instrument]
pub async fn webhook_handler(Json(payload): Json<Event>) -> impl IntoResponse {
    format!("{:?}", Event::verify_webhook_sig(payload).await.unwrap())
}
