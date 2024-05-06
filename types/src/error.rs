use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Error parse {0} to decimal, errror: {1}")]
    DecimalParseErr(String, String),
    #[error("Error negative {0} for sqrt")]
    NegativeSqrtErr(String),
    #[error("Error negative {0} for fp")]
    NegativeFpErr(String),
    #[error("Error negative/zero sigma: {0}")]
    SigmaZeroOrNegative(String),
    #[error("Error zkp error: {0}")]
    ZeroKnownledgeProofErr(String),
    #[error("Error conversion from BigInt: {0}, error: {1}")]
    BigIntConversionErr(String, String),
    #[error("Error conversion from BigRational: {0}, error: {1}")]
    BigRationalConversionErr(String, String),
    #[error("Error conversion from Decimal: {0}, Exp: {1}")]
    DecimalErr(Decimal, u32),
    #[error("Merkle tree error: {0}")]
    MerkleTreeErr(String),
    #[error("Empty merkle tree")]
    EmptyMerkleTreeErr,
}
