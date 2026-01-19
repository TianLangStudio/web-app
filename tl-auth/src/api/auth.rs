use crate::entity::auth::{create_jwt, JwtToken};
use crate::entity::user::User;
use crate::service::KeyPairService;
use crate::AuthState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use jsonwebtoken::EncodingKey;
use rboot::log::error;
use std::default::Default;
use std::sync::{Arc, RwLock};

pub(crate) async fn login(
    State(key_pair_service): State<Arc<RwLock<KeyPairService>>>,
) -> (StatusCode, Json<JwtToken>) {
    let user = User {
        id: 0,
        first_name: None,
        last_name: None,
        ..Default::default()
    };
    let encoding_key = match key_pair_service.write() {
        Ok(mut guard) => {
            let key_pair = guard.get_key_pair();
            let private_key = key_pair.get_private_key();
            EncodingKey::from_ec_pem(private_key.as_bytes())
        }
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(JwtToken::default())),
    };
    if let Err(err) = encoding_key {
        error!("Can't parse encoding key: {}", err);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(JwtToken::default()));
    }
    let encoding_key = encoding_key.unwrap();
    match create_jwt(user, &encoding_key) {
        Ok(token) => (StatusCode::OK, Json(token)),
        Err(err) => {
            error!("{:?}", err);
            (StatusCode::BAD_REQUEST, Json(JwtToken::default()))
        }
    }
}

pub(crate) async fn who_am_i(
    //token_body: Claims<TokenBody>,
    user: User,
    _auth_state: State<AuthState>,
) -> Response {
    //Json(token_body.claims).into_response()
    Json(user).into_response()
}
