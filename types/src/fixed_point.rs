use std::ops::{Add, Mul};

use num_bigint::BigInt;
use num_rational::{BigRational, Ratio};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use tracing::error;

use crate::Error;
pub trait FixedPoint:
    Eq + PartialEq + Clone + std::fmt::Debug + Mul<Output = Self> + Add<Output = Self>
{
    fn zero() -> Self;
}
pub trait FixedPointInteger: FixedPoint {
    fn fixed_from_f64(value: f64, multiplier: &Self) -> Result<Self, Error>;
}
pub trait FixedPointDecimal: FixedPoint {
    fn fixed_from_f64(value: f64) -> Result<Self, Error>;
}
impl FixedPoint for BigInt {
    fn zero() -> Self {
        BigInt::default()
    }
}
impl FixedPoint for Decimal {
    fn zero() -> Self {
        Decimal::ZERO
    }
}
impl FixedPoint for Ratio<BigInt> {
    fn zero() -> Self {
        Ratio::default()
    }
}

impl FixedPointInteger for BigInt {
    fn fixed_from_f64(value: f64, multiplier: &Self) -> Result<Self, Error> {
        let r = BigRational::from_float(value).ok_or({
            let e = Error::DecimalParseErr(value);
            error!("{:?}", e);
            e
        })? * multiplier;
        return Ok(r.round().to_integer());
    }
}

impl FixedPointDecimal for Decimal {
    fn fixed_from_f64(value: f64) -> Result<Self, Error> {
        Ok(Decimal::from_f64(value).ok_or_else(|| {
            let e = Error::DecimalParseErr(value);
            error!("{:?}", e);
            e
        })?)
    }
}

impl FixedPointDecimal for Ratio<BigInt> {
    fn fixed_from_f64(value: f64) -> Result<Self, Error> {
        let r = BigRational::from_float(value).ok_or_else(|| {
            let e = Error::DecimalParseErr(value);
            error!("{:?}", e);
            e
        })?;
        Ok(r)
    }
}
