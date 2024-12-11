use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use rboot::config::load_config;
use rboot::database::init_db_pool;
use sqlx::{PgPool};

#[tokio::main]
async fn main() {
    let config = load_config();
    let pool = init_db_pool(&config).await;
    let server_port = config
        .get_int("server.port")
        .expect("server.port is required");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/db_test", get(db_test))
        .route("/json_test", post(json_test))
        .with_state(pool);
    // run our app with hyper, listening globally on port server_port
    let addr = format!("0.0.0.0:{}", server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
async fn db_test(State(pool): State<PgPool>) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("select 'hello from postgresql'")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)
}
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Account {
    email: String,
    password: String,
}
async fn json_test(account: Json<Account>) -> Result<Json<Account>, (StatusCode, String)> {
    println!("account: {:?}", account);
    Ok(account)
}
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
