use crate::entity::auth::{JwtToken, create_jwt};
use crate::entity::user::User;
use crate::get_key_pair_service;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::EncodingKey;
use rboot::log::error;
use std::default::Default;

pub(crate) async fn login() -> (StatusCode, Json<JwtToken>) {
    let user = User {
        id: 0,
        first_name: None,
        last_name: None,
        ..Default::default()
    };
    let key_pair_service = get_key_pair_service().unwrap();
    let encoding_key = match key_pair_service.read() {
        Ok(guard) => {
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
) -> Response {
    //Json(token_body.claims).into_response()
    Json(user).into_response()
}
