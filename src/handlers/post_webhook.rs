use crate::axumlib::State as MyState;
use crate::models::report::Report;
use crate::services::post_webhook::post_webhook;
use crate::services::telegram::post;
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
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, Report> {
    let header_opt = headers
        .get("Tailscale-Webhook-Signature")
        .ok_or_else(|| eyre!("no header"))?;
    let header = header_opt
        .to_str()
        .map_err(|err| eyre!("{err}"))?
        .tap_deref(|header| info!(header, "Got header"));
    let now = Utc::now();

    let ts_secret = state.tailscale_secret;
    let events = post_webhook(header, &body, now, &ts_secret)?;
    info!(?events, "Got events");

    let tg_secret = state.telegram_secret;
    let reqwest_client = state.reqwest_client;
    let chat_id = state.chat_id;
    post(events, reqwest_client, tg_secret, chat_id).await?;
    Ok(())
}
