use config::{Config, File, FileFormat};
pub fn load_config() -> Config {
    let builder = Config::builder();
    let default_builder =
        builder.add_source(File::new("config/application.yaml", FileFormat::Yaml));
    let config = default_builder
        .clone()
        .build()
        .expect("Load config file error");
    if let Ok(active_profile) = config.get_string("profile.active") {
        let config_file_path = format!("config/application-{}.yaml", active_profile);
        let active_builder =
            default_builder.add_source(File::new(config_file_path.as_str(), FileFormat::Yaml));
        let config = active_builder
            .build()
            .unwrap_or_else(|_| panic!("Load config file {config_file_path} error"));
        return config;
    }
    config
}
