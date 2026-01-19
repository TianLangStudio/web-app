use crate::entity::auth::{AuthError, AuthState, TokenBody};
use crate::service::KeyPairService;
use axum::extract::FromRef;
use axum::{
    extract::FromRequestParts, http::{request::Parts},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::errors::{ErrorKind};
use jsonwebtoken::{decode, Algorithm, Validation};
use rboot::log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct User {
    pub id: u32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub org_id: u32,
    pub roles: Vec<String>,
}

impl FromRequestParts<AuthState> for User {
    type Rejection = AuthError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AuthState,
    ) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let key_service = Arc::<RwLock<KeyPairService>>::from_ref(state);
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        let (decoding_key, last_decoding_key) = {
            let lock = key_service
                .read()
                .map_err(|_| AuthError::InvalidDecodingKey)?;
            lock
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
