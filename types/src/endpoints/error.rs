use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error parse {0} to decimal")]
    DecimalParseErr(f64),
}
