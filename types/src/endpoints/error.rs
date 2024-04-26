use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error parse {0} to decimal")]
    DecimalParseErr(f64),
    #[error("Error negative {0} for sqrt")]
    NegativeSqrtErr(String),
    #[error("Error negative/zero sigma: {0}")]
    SigmaZeroOrNegative(String),
}
