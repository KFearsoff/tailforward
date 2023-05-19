use crate::models::report::Report;
use crate::services::post_webhook::post_webhook;
use axum::extract::Extension;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use chrono::Utc;
use color_eyre::eyre::eyre;
use secrecy::{ExposeSecret, SecretString};

#[tracing::instrument]
pub async fn webhook_handler(
    Extension(secret): Extension<SecretString>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, Report> {
    let header_opt = headers
        .get("Tailscale-Webhook-Signature")
        .ok_or_else(|| eyre!("no header"))?;
    let header = header_opt.to_str().map_err(|err| eyre!("{err}"))?;
    let now = Utc::now();
    let events = post_webhook(header, &body, now, secret.expose_secret());
    //info!(events = "{events:?}", "Got events");
    Ok(StatusCode::OK)
    // Message::post(events).await
}
