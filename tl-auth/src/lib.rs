use crate::entity::auth::AuthState;
use crate::service::KeyPairService;
use axum::Router;
use axum::routing::{get, post};
use chrono::Duration;
use rboot::config::TLConfig;
use rboot::log::{error, info};
use rboot::task_scheduler::EVERY_1_HOUR_CRON;
use std::sync::{OnceLock, RwLock};

pub mod api;
pub mod entity;
mod service;

static KEY_SERVICE_LOCK: OnceLock<RwLock<KeyPairService>> = OnceLock::new();
pub fn get_key_pair_service() -> Option<&'static RwLock<KeyPairService>> {
    KEY_SERVICE_LOCK.get()
}

pub async fn init<S>(config: &TLConfig, outer_router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let key_live_hours_num = config.get_int("jwt.encryption.key.live.hours").unwrap_or(2);
    info!("jwt encryption key live hours: {}", key_live_hours_num);
    let key_live_hours = Duration::hours(key_live_hours_num);
    let key_service = crate::service::KeyPairService::new(key_live_hours)
        .expect("Failed to initialize KeyPairService");
    let key_service_lock = RwLock::new(key_service);
    KEY_SERVICE_LOCK
        .set(key_service_lock)
        .expect("Failed to initialize KeyPairService");

    if key_live_hours_num > 0
        && let Err(err) = rboot::task_scheduler::add_job_to_scheduler(
            EVERY_1_HOUR_CRON,
            Box::new(|id, _l| {
                info!("run key pair refresh task with id {}", id);
                Box::pin(async {
                    let key_pair_guard = get_key_pair_service();
                    if key_pair_guard.is_none() {
                        info!("There is no key pair service");
                        return;
                    }
                    if let Err(err) = get_key_pair_service()
                        .unwrap()
                        .write()
                        .map(|mut guard| guard.refresh_key_pair())
                    {
                        info!("key service lock is unavailable: {}", err);
                    }
                })
            }),
        )
        .await
    {
        error!("Failed to add job to scheduler: {:?}", err);
    }
    /* let encode_key_path = config
        .get_string("jwt.encode.key.path")
        .unwrap_or("config/jwt.key".to_string());
    let encode_key = read_to_string(&encode_key_path)
        .unwrap_or_else(|_| panic!("Can't find encode key with path {}", encode_key_path));
    let encoding_key = EncodingKey::from_rsa_pem(encode_key.as_bytes())
        .unwrap_or_else(|_| panic!("Can't parse encode key with path {}", encode_key_path));
    let encoding_key = Arc::new(encoding_key);

    //decode
    let public_key = key_service.get_key_pair().get_public_key();
    let keys = vec![DecodingKey::from_ec_pem(public_key.as_bytes()).expect("Can't parse key")];
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_audience(&["https://example.com"]);
    let decoder = LocalDecoder::builder()
        .keys(keys)
        .validation(validation)
        .build()
        .unwrap();*/

    //let key_service = Arc::new(RwLock::new(key_service));
    let auth_state = AuthState {
        //encoding_key,
        //decoder: Arc::new(decoder),
       // key_service,
    };
    let auth_router = Router::new()
        .route("/api/auth/login", get(api::auth::login))
        .route("/api/auth/whoami", post(api::auth::who_am_i))
        .with_state(auth_state);
    outer_router.merge(auth_router)
}
