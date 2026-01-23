use rboot::jwt::{create_jwt, JwtToken};
use crate::entity::user::{EmailPwdCredentials, User};
use axum::Json;
use axum::http::StatusCode;

use rboot::log::{error, info};
use std::default::Default;

pub async fn login(payload: Json<EmailPwdCredentials>) -> (StatusCode, Json<JwtToken>) {
    info!("login with payload: {:?}", payload);

    let user = User {
        id: 0,
        first_name: None,
        last_name: None,
        email: Some(payload.email.clone()),
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

