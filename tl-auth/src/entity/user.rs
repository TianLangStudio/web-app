use crate::entity::auth::{AuthError, TokenBody};
use crate::get_key_pair_service;
use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, Validation, decode};
use rboot::log::info;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct User {
    pub id: u32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub org_id: u32,
    pub roles: Vec<String>,
}

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync + Clone + 'static,
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
        decode::<TokenBody>(token, &decoding_key, &validation)
            .or_else(|err| {
                info!("Token decode error: {:?}", err);
                if err.kind() != &ErrorKind::InvalidSignature {
                    return Err(err);
                }
                //try to decode the token with the last public key.
                if let Some(last_decoding_key) = last_decoding_key {
                    decode::<TokenBody>(bearer.token(), &last_decoding_key, &validation)
                } else {
                    Err(err)
                }
            })
            .map(|token_data| token_data.claims.user)
            .map_err(|_| AuthError::InvalidToken)
    }
}
