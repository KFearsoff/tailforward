// use once_cell::sync::Lazy;
// use std::net::{SocketAddr, TcpListener};
// use tailforward_cfg::Config;
//
// static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(|| Config::default());
//
// #[tokio::test]
// async fn ping_works() {
//     // Arrange
//     let addr = TcpListener::bind("127.0.0.1:0")
//         .unwrap()
//         .local_addr()
//         .unwrap();
//     spawn_app(&GLOBAL_CONFIG, &addr);
//     let client = reqwest::Client::new();
//
//     // Act
//     let response = client
//         .get(format!("http://127.0.0.1:{}/ping", addr.port()))
//         .send()
//         .await
//         .expect("Failed to execute request");
//
//     // Assert
//     assert!(response.status().is_success());
//     assert_eq!(Some(0), response.content_length());
// }
//
// fn spawn_app(config: &'static Config, address: &SocketAddr) {
//     let app = tailforward::setup_app(config).unwrap();
//     let server = axum::Server::bind(address).serve(app.into_make_service());
//     let _ = tokio::spawn(server);
// }
