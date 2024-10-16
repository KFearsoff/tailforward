use once_cell::sync::Lazy;
use std::future::IntoFuture;
use tailforward::config::{new_config_with_secrets, Application};
use tokio::net::TcpListener;

static GLOBAL_CONFIG: Lazy<Application> = Lazy::new(|| {
    new_config_with_secrets("tail".to_owned().into(), "tele=gram".to_owned().into()).unwrap()
});

#[tokio::test]
async fn ping_works() {
    // Arrange
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    spawn_app(GLOBAL_CONFIG.to_owned(), listener);
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("http://127.0.0.1:{}/ping", addr.port()))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app(config: Application, listener: TcpListener) {
    let app = tailforward::setup_app(config).unwrap();
    let server = axum::serve(listener, app.into_make_service()).into_future();
    let _ = tokio::spawn(server);
}
