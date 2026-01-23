use rboot::jwt::{create_jwt, JwtToken};
use crate::entity::user::User;
use axum::Json;
use axum::http::StatusCode;

use rboot::log::error;
use std::default::Default;

pub async fn login() -> (StatusCode, Json<JwtToken>) {
    let user = User {
        id: 0,
        first_name: None,
        last_name: None,
        ..Default::default()
    };
    match create_jwt(user) {
        Ok(token) => (StatusCode::OK, Json(token)),
        Err(err) => {
            error!("{:?}", err);
            (StatusCode::BAD_REQUEST, Json(JwtToken::default()))
        }
    }
}

