use crate::models::report::Report;
use crate::services::post_webhook::post_webhook;
use crate::services::telegram::post;
use axum::extract::Extension;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use chrono::Utc;
use color_eyre::eyre::eyre;
use secrecy::{ExposeSecret, SecretString};
use tracing::info;

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
    info!(header, "Got header");
    let now = Utc::now();
    let events = post_webhook(header, &body, now, secret.expose_secret())?;
    info!(events = "{events:?}", "Got events");
    post(events).await?;
    Ok(())
}