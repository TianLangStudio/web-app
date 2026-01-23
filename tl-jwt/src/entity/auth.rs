use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::{Json, RequestPartsExt};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::TypedHeader;
use headers::Authorization;
use headers::authorization::Bearer;
use jsonwebtoken::{Algorithm, Validation, decode};
use jsonwebtoken::errors::ErrorKind;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::json;
use tl_log::{info};
use crate::get_key_pair_service;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Claims<U> {
    pub user: U,
    pub create_at: i64, //create at time
    pub exp: i64,
}
impl<S, U> FromRequestParts<S> for Claims<U>
where
    S: Send + Sync + Clone + 'static,
    U: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    type Rejection = AuthError;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let key_service = get_key_pair_service().ok_or(AuthError::InvalidDecodingKey)?;
        let key_service = key_service
            .read()
            .map_err(|_| AuthError::InvalidDecodingKey)?;
        let (decoding_key, last_decoding_key) = {
            key_service
                .get_decoding_keys()
                .map_err(|_| AuthError::InvalidDecodingKey)?
        };
        // Decode the user data
        let token = bearer.token();
        let validation = Validation::new(Algorithm::ES256);
        decode::<Claims<U>>(token, &decoding_key, &validation)
            .or_else(|err| {
                info!("Token decode error: {:?}", err);
                if err.kind() != &ErrorKind::InvalidSignature {
                    return Err(err);
                }
                //try to decode the token with the last public key.
                if let Some(last_decoding_key) = last_decoding_key {
                    decode::<Claims<U>>(bearer.token(), &last_decoding_key, &validation)
                } else {
                    Err(err)
                }
            })
            .map(|token_data| token_data.claims)
            .map_err(|_| AuthError::InvalidToken)
    }
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