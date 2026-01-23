use axum::routing::{get, post};
use axum::{Json, Router};
use rboot::jwt::Claims;
use rboot::log::info;
use user_center::api::auth;
use user_center::entity::user::User;


#[derive(Clone)]
struct AppState {}
#[tokio::main]
async fn main() {
    rboot::log::init();
    let config =
        rboot::config::load_config().unwrap_or_else(|err| panic!("load config failed: {:?}", err));
    rboot::jwt::init(&config).await.unwrap_or_else(|err| panic!("init TlAuth failed: {:?}", err));
    let server_host = config
        .get_string("server.host")
        .unwrap_or("0.0.0.0".to_string());
    let server_port = config.get::<u16>("server.port").unwrap_or(3000);
    info!("Server is starting on: {}:{}", server_host, server_port);

    let router = Router::new()
        .route("/", get(index))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/whoami", post(whoami))
        .with_state(AppState {});


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
async fn whoami(claims: Claims<User>) -> Json<User> {
    claims.user.into()
}
