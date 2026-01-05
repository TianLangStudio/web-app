use std::default::Default;
use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use jsonwebtoken::EncodingKey;
use crate::entity::auth::{create_jwt, JwtToken};
use crate::entity::user::User;

pub async fn login(State(encoding_key): State<Arc<EncodingKey>>) -> (StatusCode, Json<JwtToken>) {
    let user = User {
        id: 0,
        first_name: None,
        last_name: None,
        .. Default::default()
    };

    if let Ok(token) = create_jwt(&user, &encoding_key) {
        (StatusCode::OK, Json(token))
    }else {
        (StatusCode::BAD_REQUEST, Json(JwtToken::default()))
    }
}