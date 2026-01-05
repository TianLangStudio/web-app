use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use crate::entity::user::User;

// login with email and password
pub struct EmailPwdCredentials {
    pub email: String,
    pub password: String,
    pub org_id: u32,
}

#[derive(Serialize)]
pub struct TokenBody<'a> {
    pub user: &'a User,
    pub create_at: i64,//create at time
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct JwtToken {
    pub token: String,
    pub token_type: String,
}
pub fn create_jwt(user: &User, encoding_key: &EncodingKey) -> Result<JwtToken, String> {
    let token_body = TokenBody {
        user,
        create_at: chrono::Utc::now().timestamp(),
    };
    let header = Header::new(Algorithm::RS256);
    encode::<TokenBody>(&header, &token_body, encoding_key)
        .map(|token|{
            JwtToken {
                token,
                token_type: "Bearer".to_string()
            }
        })
        .map_err(|e| format!("JWT encoding error: {}", e))
}
