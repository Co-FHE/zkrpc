#[cfg(test)]
mod tests {
    use tracing_subscriber::fmt;

    use crate::{Config, EnvironmentKind, PersistableConfig, BASE_CONFIG};

    use std::{fs, path::Path};

    #[test]
    fn test_config() {
        // let _subscriber = fmt::Subscriber::builder()
        //     .with_max_level(tracing::Level::INFO)
        //     .try_init();

        let config = Config::new().unwrap();
        assert_eq!(BASE_CONFIG.env, EnvironmentKind::Testing);
        assert_eq!(config.log.log_dir, Path::new("logs"));
    }

    #[test]
    fn test_persistable_config() {
        let _subscriber = fmt::Subscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .try_init();
        let config = Config::default();
        // get current file path
        let path = "test_config/config-for-test-save-load.yaml";
        let _ = fs::remove_file(path);
        config.save_config(path).unwrap();
        let loaded_config = Config::load_config(path).unwrap();
        assert_eq!(config.log.log_dir, Path::new("logs"));
        assert_eq!(config, loaded_config);
        // fs::remove_file(path).unwrap();
    }
}
