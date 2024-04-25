use lazy_static::lazy_static;
use num_bigint::{BigInt, BigUint, ToBigInt};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub const COORDINATE_PRECISION_BIGINT: u32 = 5;
pub const RSPR_PRECISION_BIGINT: u32 = 8;

pub const SIGMA_RANGE: Decimal = dec!(3.0);
pub const SIGMA: Decimal = dec!(0.1);
