use crate::models::report::Result;
use axum::response::IntoResponse;

#[tracing::instrument]
pub async fn ping_handler() -> Result<impl IntoResponse> {
    Ok(())
}
