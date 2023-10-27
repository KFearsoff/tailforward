use crate::models::report::Result;
use crate::models::Header;
use crate::services::post_webhook::post_webhook;
use crate::services::telegram::post;
use crate::State as MyState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
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

    let header: Header = headers
        .get(header_name)
        .ok_or_else(|| eyre!("No header {header_name} received, the request is not coming from Tailscale or the format has changed"))?
        .to_str()
        .map_err(|err| eyre!("Header {header_name} contains non-ASCII characters, Tailscale sends ASCII only: {err}"))?
        .tap_deref(|header_val| info!(header_val, "Received header {header_name}"))
        .parse()
        .map_err(|err| eyre!("Header {header_name} is invalid: {err}"))?;

    let ts_secret = state.settings.tailscale_secret;
    let events = post_webhook(header, &body, &ts_secret)?;
    info!(?events, "Got events");

    let tg_secret = state.settings.telegram_secret;
    let reqwest_client = state.reqwest_client;
    let chat_id = state.settings.base.chat_id;
    post(events, reqwest_client, tg_secret, chat_id).await?;
    Ok(())
}
