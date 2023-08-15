use crate::models::report::Report;
use axum::response::IntoResponse;

#[tracing::instrument]
pub async fn ping_handler() -> Result<impl IntoResponse, Report> {
    Ok(())
}
