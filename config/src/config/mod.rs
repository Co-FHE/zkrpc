use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error accessing {0}: {1}")]
    IO(String, #[source] std::io::Error),
    #[error("Error (de)serializing {0}: {1}")]
    Yaml(String, #[source] serde_yaml::Error),
    #[error("Config is missing expected value: {0}")]
    Missing(&'static str),
}
