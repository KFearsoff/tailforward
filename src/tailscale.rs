struct Event {
    timestamp: String,
    version: i8,
    r#type: String,
    tailnet: String,
    message: String,
    data: String,
}

#[warn(clippy::unused_async)]
pub async fn verify_webhook_sig() {
    unimplemented!();
}

#[warn(clippy::unused_async)]
pub async fn parse_sig_header() {
    unimplemented!();
}
