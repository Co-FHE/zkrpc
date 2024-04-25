#![forbid(unsafe_code)]

use num_bigint::BigInt;
use rust_decimal::Decimal;
use types::{FixedPoint, Satellite, Terminal};

mod error;
use error::*;
mod mock;
use mock::*;

pub trait DaLayerTrait {
    async fn new(cfg: &config::config::DaLayerConfig) -> Result<Self, Error>
    where
        Self: Sized;
    async fn fetch_satellite_with_terminals_block_from_to(
        &self,
        satellite_address: &str,
        block_height_from: u64,
        block_height_to: u64,
    ) -> Result<Satellite<Decimal>, Error>;
}
