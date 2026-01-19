use crate::entity::auth::AuthState;
use axum::routing::get;
use axum::Router;
use chrono::Duration;
use rboot::config::TLConfig;
use std::sync::{Arc, RwLock};

pub mod api;
pub mod entity;
mod service;

pub fn init<S>(config: &TLConfig, outer_router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let key_live_hours = config.get_int("jwt.encryption.key.live.hours").unwrap_or(2);
    let key_live_hours = Duration::hours(key_live_hours);
    let key_service = crate::service::Es256KeyPairService::new(key_live_hours)
        .expect("Failed to initialize Es256KeyPairService");
   /* let encode_key_path = config
        .get_string("jwt.encode.key.path")
        .unwrap_or("config/jwt.key".to_string());
    let encode_key = read_to_string(&encode_key_path)
        .unwrap_or_else(|_| panic!("Can't find encode key with path {}", encode_key_path));
    let encoding_key = EncodingKey::from_rsa_pem(encode_key.as_bytes())
        .unwrap_or_else(|_| panic!("Can't parse encode key with path {}", encode_key_path));
    let encoding_key = Arc::new(encoding_key);

    //decode
    let public_key = key_service.get_key_pair().get_public_key();
    let keys = vec![DecodingKey::from_ec_pem(public_key.as_bytes()).expect("Can't parse key")];
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_audience(&["https://example.com"]);
    let decoder = LocalDecoder::builder()
        .keys(keys)
        .validation(validation)
        .build()
        .unwrap();*/

    let key_service = Arc::new(RwLock::new(key_service));
    let auth_state = AuthState {
        //encoding_key,
        //decoder: Arc::new(decoder),
        key_service,
    };
    let auth_router = Router::new()
        .route("/api/auth/login", get(api::auth::login))
        .route("/api/auth/whoami", get(api::auth::who_am_i))
        .with_state(auth_state);
    outer_router.merge(auth_router)
}
