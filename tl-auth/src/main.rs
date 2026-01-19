use axum::Router;
use axum::routing::get;
use rboot::log::info;
use tl_auth::init;
#[derive(Clone)]
struct AppState {}
#[tokio::main]
async fn main() {
    rboot::log::init_log();
    let config = rboot::config::load_config();
    let server_host = config
        .get_string("server.host")
        .unwrap_or("0.0.0.0".to_string());
    let server_port = config.get::<u16>("server.port").unwrap_or(3000);
    info!("Server is starting on: {}:{}", server_host, server_port);

    let router = Router::new().route("/", get(index)).with_state(AppState {});

    let router = init(&config, router);
    let listener = tokio::net::TcpListener::bind(format!("{server_host}:{server_port}"))
        .await
        .expect("bind port failed");
    axum::serve(listener, router)
        .await
        .expect("launch server failed");
}

async fn index() -> &'static str {
    "developed by tianlang.tech"
}
