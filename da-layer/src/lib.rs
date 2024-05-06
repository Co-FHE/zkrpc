#![forbid(unsafe_code)]

use rust_decimal::Decimal;
use std::future::Future;
use types::Satellite;
mod error;
use error::*;
mod mock;
pub use mock::*;

pub trait DaLayerTrait {
    fn new(cfg: &config::DaLayerConfig) -> impl Future<Output = Result<Self, error::Error>>
    where
        Self: Sized;
    fn fetch_satellite_with_terminals_block_from_to(
        &self,
        satellite_address: &str,
        block_height_from: u64,
        block_height_to: u64,
    ) -> impl std::future::Future<Output = Result<Vec<(usize, Satellite<Decimal>)>, Error>>;
}
