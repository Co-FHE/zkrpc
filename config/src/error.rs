use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error accessing {0}: {1}")]
    IOErr(String, #[source] std::io::Error),
    #[error("Error (de)serializing {0}: {1}")]
    YamlErr(String, #[source] serde_yaml::Error),
    #[error("Config is missing expected value: {0}")]
    MissingErr(&'static str),
    #[error("Config file already exists: {0}")]
    FileExistsErr(&'static str),
    // home dir not found
    #[error("Home directory not found")]
    HomeDirNotFound,
}
