pub use tracing::{debug, error, info, trace, warn, span, Level};
pub fn init() {
    tracing_subscriber::fmt().init();
    let version: &str = env!("CARGO_PKG_VERSION");
    info!(r#"
    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
    $$$$$$$$ rboot developed by FusionZhu@tianlang.tech $$$$$$$$
    $$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$$
    version:{version}
    "#);
    let test_logs = span!(Level::INFO, "Test Logs");
    let _enter = test_logs.enter();
    debug!("debug level log");
    trace!("trace level log");
    info!("info level log");
    warn!("warn level log");
    error!("error level log");
}