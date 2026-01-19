use chrono::{Duration, Utc};
use jsonwebtoken::DecodingKey;
use p256::elliptic_curve::rand_core::OsRng;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey};
use p256::{
    SecretKey,
    ecdsa::{SigningKey, VerifyingKey},
};
use pem_rfc7468::LineEnding;

use rboot::log::warn;

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct Es256KeyPair {
    #[serde(skip_serializing)]
    private: String,
    public: String,
    timestamp: i64,
}

impl Es256KeyPair {
    fn new() -> anyhow::Result<Self> {
        let (private, public) = generate_es256_key()?;
        let key_pair = Es256KeyPair {
            private,
            public,
            timestamp: Utc::now().timestamp_millis(),
        };
        Ok(key_pair)
    }

    pub fn get_private_key(&self) -> &str {
        self.private.as_str()
    }
    pub fn get_public_key(&self) -> &str {
        self.public.as_str()
    }
}
pub(crate) struct Es256KeyPairService {
    key_pair: Es256KeyPair,
    last_key_pair: Option<Es256KeyPair>,
    key_lifetime: Duration,
}
pub type KeyPairService = Es256KeyPairService;

impl Es256KeyPairService {
    pub fn new(lifetime: Duration) -> anyhow::Result<Self> {
        let key_pair = Es256KeyPair::new()?;
        Ok(Self {
            key_pair,
            last_key_pair: None,
            key_lifetime: lifetime,
        })
    }
    pub fn get_public_key(&self) -> &str {
        self.key_pair.get_public_key()
    }
    pub fn get_last_public_key(&self) -> Option<&str> {
        self.last_key_pair.as_ref().map(|k| k.public.as_str())
    }

    pub fn get_public_keys(&self) -> (&str, Option<&str>) {
        (self.get_public_key(), self.get_last_public_key())
    }

    pub fn get_decoding_keys(&self) -> anyhow::Result<(DecodingKey, Option<DecodingKey>)> {
        let (public_key, last_public_key) = self.get_public_keys();
        let decoding_key = DecodingKey::from_ec_pem(public_key.as_bytes())?;
        let last_decoding_key = match last_public_key {
            Some(public_key) => DecodingKey::from_ec_pem(public_key.as_bytes()).ok(),
            None => None,
        };
        Ok((decoding_key, last_decoding_key))
    }
    pub fn get_key_pair(&mut self) -> &Es256KeyPair {
        if Utc::now().timestamp_millis()
            > self.key_lifetime.num_milliseconds() + self.key_pair.timestamp
        {
            self.last_key_pair.replace(self.key_pair.clone());
            match Es256KeyPair::new() {
                Ok(key_pair) => {
                    self.key_pair = key_pair;
                }
                Err(e) => {
                    warn!("error generate new key pair: {}", e);
                }
            }
        }
        &self.key_pair
    }
/*
    pub fn get_key_pair_with_timestamp(&self, timestamp: i64) -> Option<&Es256KeyPair> {
        if self.key_pair.timestamp == timestamp {
            return Some(&self.key_pair);
        } else if self.last_key_pair.is_some()
            && self.last_key_pair.as_ref().unwrap().timestamp == timestamp
        {
            return Some(self.last_key_pair.as_ref().unwrap());
        }

        None
    }*/
}

fn generate_es256_key() -> anyhow::Result<(String, String)> {
    // 1. Generate secure random private key (32 bytes)
    let secret_key = SecretKey::random(&mut OsRng);
    // 2. Create signing key (for ES256 = ECDSA/P-256/SHA-256)
    let signing_key: SigningKey = secret_key.into();
    // 3. Derive verifying (public) key
    let verifying_key: VerifyingKey = VerifyingKey::from(&signing_key);
    // PEM PKCS#8 private key (very common for JWT libraries)
    let pem_private = signing_key.to_pkcs8_pem(LineEnding::LF)?.to_string();

    // PEM SubjectPublicKeyInfo (SPKI) public key
    let pem_pub = verifying_key
        .to_public_key_der()?
        .to_pem("PUBLIC KEY", LineEnding::LF)?;

    Ok((pem_private, pem_pub))
}

#[cfg(test)]
mod tests {
    use crate::service::generate_es256_key;
    use rboot::log::info;

    #[test]
    fn test_generate_es256_key() {
        let (private, public) = generate_es256_key().unwrap();
        info!("private: {:?} public:{:?}", private, public);
    }
}
