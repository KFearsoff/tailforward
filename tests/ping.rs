use once_cell::sync::Lazy;
use tailforward_cfg::Config;

static GLOBAL_CONFIG: Lazy<Config> =
    Lazy::new(|| tailforward::config::new_config().expect("Failed to setup config"));

#[tokio::test]
async fn ping_works() {
    // Arrange
    spawn_app(&GLOBAL_CONFIG).await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:33010/ping")
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app(config: &'static Config) {
    let app = tailforward::setup_app(config).await.unwrap();
    let server = axum::Server::bind(&config.address).serve(app.into_make_service());
    let _ = tokio::spawn(server);
}
