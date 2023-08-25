use crate::models::report::Result;
use crate::services::post_webhook::post_webhook;
use crate::services::telegram::post;
use crate::State as MyState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use chrono::Utc;
use color_eyre::eyre::eyre;
use tap::Tap;
use tracing::info;

#[tracing::instrument]
pub async fn webhook_handler(
    State(state): State<MyState>,
    // TODO: use typed header?
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse> {
    let header_name = "Tailscale-Webhook-Signature";

    let header_opt = headers
        .get(header_name)
        .ok_or_else(|| eyre!("No header {header_name} received, the request is not coming from Tailscale or the format has changed"))?;
    let header_val = header_opt
        .to_str()
        .map_err(|err| eyre!("Header {header_name} contains non-ASCII characters, Tailscale sends ASCII only: {err}"))?
        .tap_deref(|header_val| info!(header_val, "Received header {header_name}"));

    let now = Utc::now();

    let ts_secret = state.tailscale_secret;
    let events = post_webhook(header_val, &body, now, &ts_secret)?;
    info!(?events, "Got events");

    let tg_secret = state.telegram_secret;
    let reqwest_client = state.reqwest_client;
    let chat_id = state.chat_id;
    post(events, reqwest_client, tg_secret, chat_id).await?;
    Ok(())
}
