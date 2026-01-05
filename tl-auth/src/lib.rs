use std::fs::read_to_string;
use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use axum::extract::FromRef;
use jsonwebtoken::EncodingKey;
use rboot::config::TLConfig;

pub mod entity;
pub mod api;

#[derive(Clone, FromRef)]
pub struct AuthState {
    pub encoding_key: Arc<EncodingKey>
}
pub fn init<S>(config: &TLConfig, outer_router: Router<S>)
               -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let encode_key_path = config.get_string("jwt.encode.key.path").unwrap_or("config/jwt.key".to_string());
    let encode_key = read_to_string(&encode_key_path)
        .unwrap_or_else(|_| panic!("Can't find encode key with path {}", encode_key_path));
    let encoding_key = EncodingKey::from_rsa_pem(encode_key.as_bytes())
        .unwrap_or_else(|_|panic!("Can't parse encode key with path {}", encode_key_path));
    let encoding_key = Arc::new(encoding_key);
    let auth_state = AuthState { encoding_key };
    let auth_router = Router::new()
        .route("/api/auth/login", get(api::auth::login))
        .with_state(auth_state);
    outer_router.merge(auth_router)
}