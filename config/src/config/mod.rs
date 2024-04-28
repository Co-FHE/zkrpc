mod base_config;
pub use base_config::*;
use lazy_static;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
mod error;
pub use error::*;
mod log_config;
pub use log_config::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
mod rpc_config;
pub use rpc_config::*;
mod da_layer_config;
pub use da_layer_config::*;
mod pox_config;
pub use pox_config::*;
mod constants;
pub use constants::*;
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub log: LogConfig,
    pub rpc: RpcConfig,
    pub da_layer: DaLayerConfig,
    pub pox: PoxConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            log: LogConfig::default(),
            rpc: RpcConfig::default(),
            da_layer: DaLayerConfig::default(),
            pox: PoxConfig::default(),
        }
    }
}
lazy_static::lazy_static!(
    pub static ref BASE_CONFIG: BaseConfig = BaseConfig::default();
);
impl Config {
    pub fn new() -> Result<Self, Error> {
        let path = BASE_CONFIG.root_path.join("config").join("config.yaml");
        // println!(
        //     "{}: loading config from: {}",
        //     "Config Info".green(),
        //     path.().to_string().blue()
        // );
        // if path exists, load it
        let config = if path.exists() {
            Self::load_config(&path)?
        } else {
            Self::default().save_config(&path)?.to_owned()
        };

        Ok(config)
    }
}

pub trait PersistableConfig: Serialize + DeserializeOwned {
    fn load_config<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut file = File::open(&path)
            .map_err(|e| Error::IOErr(path.as_ref().to_str().unwrap().to_string(), e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::IOErr(path.as_ref().to_str().unwrap().to_string(), e))?;
        Self::parse(&contents)
    }

    fn save_config<P: AsRef<Path>>(&self, output_file: P) -> Result<&Self, Error> {
        let contents = serde_yaml::to_string(&self)
            .map_err(|e| Error::YamlErr(output_file.as_ref().to_str().unwrap().to_string(), e))?;
        if let Some(dir) = output_file.as_ref().parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| Error::IOErr(output_file.as_ref().to_str().unwrap().to_string(), e))?;
        }
        let mut file = File::create_new(output_file.as_ref())
            .map_err(|e| Error::IOErr(output_file.as_ref().to_str().unwrap().to_string(), e))?;
        file.write_all(&contents.as_bytes())
            .map_err(|e| Error::IOErr(output_file.as_ref().to_str().unwrap().to_string(), e))?;
        Ok(self)
    }

    fn parse(serialized: &str) -> Result<Self, Error> {
        serde_yaml::from_str(serialized).map_err(|e| Error::YamlErr("config".to_string(), e))
    }
}

impl<T: ?Sized> PersistableConfig for T where T: Serialize + DeserializeOwned {}

#[cfg(test)]
mod tests {
    use tracing_subscriber::fmt;

    use super::*;
    use std::fs;

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
        let path = "src/config/test_config/config-for-test-save-load.yaml";
        let _ = fs::remove_file(path);
        config.save_config(path).unwrap();
        let loaded_config = Config::load_config(path).unwrap();
        assert_eq!(config.log.log_dir, Path::new("logs"));
        assert_eq!(config, loaded_config);
        // fs::remove_file(path).unwrap();
    }
}
