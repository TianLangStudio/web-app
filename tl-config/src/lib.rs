use config::{Case, Config, ConfigError, Environment, File, FileFormat};
pub type TLConfig = Config;
pub type TLConfigError = ConfigError;
static ENVIRONMENT_PREFIX: &str = "tl";
pub fn load_config() -> Result<TLConfig, TLConfigError> {
    let builder = Config::builder();
    let environment = Environment::with_prefix(ENVIRONMENT_PREFIX).separator("_").convert_case(Case::Lower);
    let mut builder =
        builder.add_source(
            File::new("config/application.yaml", FileFormat::Yaml).required(false)
        ).add_source(environment);
    let config = builder
        .clone()
        .build()?;
    if let Ok(active_profile) = config.get::<&str>("profile.active") {
        let config_file_path = format!("config/application-{}.yaml", active_profile);
        builder = builder.add_source(File::new(config_file_path.as_str(), FileFormat::Yaml));
    }
    let app_name = config.get::<&str>("environment.prefix").unwrap_or(ENVIRONMENT_PREFIX);
    builder
        .add_source(Environment::with_prefix(app_name).separator("_").convert_case(Case::Lower))
        .build()
}


#[cfg(test)]
mod tests {
    use std::env;
    use super::*;
    #[test]
    fn test_load_config() {
        let config = load_config().unwrap();
        assert_eq!(config.get::<String>("name").unwrap(), "RBoot");
        assert_eq!(config.get::<String>("user.name").unwrap(), "Fusion Zhu");
        assert_eq!(config.get::<u32>("user.age").unwrap(), 28);
        assert_eq!(config.get::<String>("user.address.city").unwrap(), "ShenZhen");
    }


    #[test]
    fn test_load_config_from_env() {
        let user_name = "FusionZhu@tianlang.tech";
        unsafe { env::set_var("tl_user_name", user_name); }
        let config = load_config().unwrap();
        assert_eq!(config.get::<String>("user.name").unwrap(), user_name);
    }
}