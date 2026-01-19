use crate::entity::user::User;
use crate::service::Es256KeyPairService;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, RwLock};

#[derive(Clone, FromRef)]
pub(crate) struct AuthState {
   // pub encoding_key: Arc<EncodingKey>,
    pub key_service: Arc<RwLock<Es256KeyPairService>>,
   // pub decoder: Decoder<TokenBody>,
}

// login with email and password
pub struct EmailPwdCredentials {
    pub email: String,
    pub password: String,
    pub org_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenBody {
    pub user: User,
    pub create_at: i64, //create at time
    pub exp: i64,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct JwtToken {
    pub token: String,
    pub token_type: String,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InvalidDecodingKey,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AuthError::InvalidDecodingKey => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Invalid decoding key")
            }
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
pub fn create_jwt(user: User, encoding_key: &EncodingKey) -> Result<JwtToken, String> {
    let token_body = TokenBody {
        user,
        create_at: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp(),
    };
    let header = Header::new(Algorithm::ES256);
    encode::<TokenBody>(&header, &token_body, encoding_key)
        .map(|token| JwtToken {
            token,
            token_type: "Bearer".to_string(),
        })
        .map_err(|e| format!("JWT encoding error: {}", e))
}
