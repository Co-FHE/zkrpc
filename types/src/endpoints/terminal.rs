use config::config::{COORDINATE_PRECISION_BIGINT, RSPR_PRECISION_BIGINT};
use lazy_static::lazy_static;
use num_bigint::{BigInt, BigUint, ToBigInt};
use rust_decimal::Decimal;

use crate::{Error, FixedPoint, FixedPointDecimal, FixedPointInteger};

lazy_static! {
    static ref COORDINATE_MULTIPLIER_BIGINT: BigInt = BigUint::from(10u32)
        .pow(COORDINATE_PRECISION_BIGINT)
        .to_bigint()
        .unwrap();
    static ref RSPR_MULTIPLIER_BIGINT: BigInt = BigUint::from(10u32)
        .pow(RSPR_PRECISION_BIGINT)
        .to_bigint()
        .unwrap();
}

pub struct Terminal<T: FixedPoint> {
    pub address: String,
    pub x: T,
    pub y: T,
    pub alpha: Alpha<T>,
}
pub struct Alpha<T: FixedPoint> {
    pub rspr: T,
}
impl<T: FixedPoint> Alpha<T> {
    pub fn new(rspr: T) -> Self {
        Self { rspr }
    }
}
impl<T: FixedPoint> Terminal<T> {
    pub fn new(address: String, x: T, y: T, alpha: Alpha<T>) -> Self {
        Self {
            address,
            x,
            y,
            alpha,
        }
    }
}

impl Alpha<BigInt> {
    pub fn new_from_f64(rspr: f64) -> Result<Self, Error> {
        Ok(Self {
            rspr: BigInt::fixed_from_f64(rspr, &RSPR_MULTIPLIER_BIGINT)?,
        })
    }
}
impl Terminal<BigInt> {
    pub fn new_from_f64(address: String, x: f64, y: f64, alpha: f64) -> Result<Self, Error> {
        Ok(Self {
            address,
            x: BigInt::fixed_from_f64(x, &COORDINATE_MULTIPLIER_BIGINT)?,
            y: BigInt::fixed_from_f64(y, &COORDINATE_MULTIPLIER_BIGINT)?,
            alpha: Alpha::<BigInt>::new_from_f64(alpha)?,
        })
    }
}

impl Alpha<Decimal> {
    pub fn new_from_f64(rspr: f64) -> Result<Self, Error> {
        Ok(Self {
            rspr: Decimal::fixed_from_f64(rspr)?,
        })
    }
}
impl Terminal<Decimal> {
    pub fn new_from_f64(address: String, x: f64, y: f64, alpha: f64) -> Result<Self, Error> {
        Ok(Self {
            address,
            x: Decimal::fixed_from_f64(x)?,
            y: Decimal::fixed_from_f64(y)?,
            alpha: Alpha::<Decimal>::new_from_f64(alpha)?,
        })
    }
}
