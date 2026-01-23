use crate::service::KeyPairService;
use chrono::Duration;
use tl_config::TLConfig;
use tl_log::{error, info, warn};
use tl_task::EVERY_1_HOUR_CRON;
use std::sync::{OnceLock, RwLock};
use anyhow::anyhow;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;


mod entity;
mod service;

pub use entity::auth::Claims;
pub use entity::auth::JwtToken;
static KEY_SERVICE_LOCK: OnceLock<RwLock<KeyPairService>> = OnceLock::new();
pub (crate) fn get_key_pair_service() -> Option<&'static RwLock<KeyPairService>> {
    KEY_SERVICE_LOCK.get()
}

pub fn create_jwt<T>(user: T) -> anyhow::Result<JwtToken>
where
    T: Serialize,
{
    let key_pair_service = get_key_pair_service().unwrap();
    let encoding_key = key_pair_service.read()
        .map_err(|err|anyhow!("{:?}", err))
        .and_then(|guard| {
            let key_pair = guard.get_key_pair();
            let private_key = key_pair.get_private_key();
            EncodingKey::from_ec_pem(private_key.as_bytes())
                .map_err(|err| anyhow!("load encoding key failed: {}", err))
        })
        .map_err(|err| anyhow!("create encoding key failed: {}", err))?;


    let token_body = Claims {
        user,
        create_at: chrono::Utc::now().timestamp(),
        exp: (chrono::Utc::now() + Duration::days(1)).timestamp(),
    };
    let header = Header::new(Algorithm::ES256);
    encode::<Claims<T>>(&header, &token_body, &encoding_key)
        .map(|token| JwtToken {
            token,
            token_type: "Bearer".to_string(),
        }).map_err(|err| anyhow!(err))
}

pub async fn init(config: &TLConfig) -> anyhow::Result<()>
{
    if KEY_SERVICE_LOCK.get().is_some() {
        warn!("TlAuth has been initialed");
        return Ok(());
    }

    let key_live_hours_num = config.get_int("jwt.encryption.key.live.hours").unwrap_or(2);
    info!("jwt encryption key live hours: {}", key_live_hours_num);
    let key_live_hours = Duration::hours(key_live_hours_num);
    let key_service = KeyPairService::new(key_live_hours)
        .map_err(|err| anyhow!("Failed to initialize KeyPairService {:?}", err))?;
    let key_service_lock = RwLock::new(key_service);

    KEY_SERVICE_LOCK
        .set(key_service_lock)
        .map_err(|err|anyhow!("Failed to initialize KeyPairService {:?}", err))?;

    if key_live_hours_num > 0
        && let Err(err) = tl_task::add_job_to_scheduler(
            EVERY_1_HOUR_CRON,
            Box::new(|id, _l| {
                info!("run key pair refresh task with id {}", id);
                Box::pin(async {
                    let key_pair_guard = get_key_pair_service();
                    if let Err(err) = key_pair_guard
                        .ok_or(anyhow!("There is no key pair service"))
                        .and_then(|guard| {
                            guard.write().map_err(|err|anyhow!("{:?}", err))
                        })
                        .and_then(|mut guard| {
                            guard.refresh_key_pair()
                        }) {
                        warn!("refresh key pair error: {}", err);
                    }
                })
            }),
        )
        .await
    {
        error!("Failed to add job to scheduler: {:?}", err);
    };
    info!("TlAuth is initialed successfully");
    Ok(())
}
