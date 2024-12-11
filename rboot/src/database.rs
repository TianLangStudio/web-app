use config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;

pub async fn init_db_pool(config: &Config) -> Pool<Postgres> {
    let db_host = config
        .get_string("database.host")
        .expect("database.host is required");
    let db_port = config
        .get_int("database.port")
        .expect("database.port is required");
    let db_user = config
        .get_string("database.username")
        .expect("database.username is required");
    let db_password = config
        .get_string("database.password")
        .expect("database.password is required");
    let db_name = config
        .get_string("database.name")
        .expect("database.name is required");
    let db_connection_str =
        format!("postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");
    
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(10))
        .max_connections(5)
        .connect(&db_connection_str)
        .await
        .expect("Connect to the Database fail, please check the link information")
}
