use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::Report as EyreReport;
use std::result;
use tracing::error;

pub type Result<T, E = Report> = result::Result<T, E>;
pub struct Report(EyreReport);

impl From<EyreReport> for Report {
    fn from(err: EyreReport) -> Self {
        Self(err)
    }
}

impl IntoResponse for Report {
    fn into_response(self) -> Response {
        error!("Internal server error: {:?}", self.0);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
