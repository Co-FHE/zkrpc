use std::ops::{Add, Div, Mul, Rem, Sub};

use num_bigint::{BigInt, BigUint, Sign};
use num_rational::{BigRational, Ratio};
use num_traits::*;
use rust_decimal::{prelude::FromPrimitive, Decimal, MathematicalOps};
use tracing::error;

use crate::Error;

pub trait FixedPointOps<Rhs = Self, Output = Self>:
    Add<Rhs, Output = Output>
    + Sub<Rhs, Output = Output>
    + Mul<Rhs, Output = Output>
    + Div<Rhs, Output = Output>
    + Rem<Rhs, Output = Output>
{
}

impl<T, Rhs, Output> FixedPointOps<Rhs, Output> for T where
    T: Add<Rhs, Output = Output>
        + Sub<Rhs, Output = Output>
        + Mul<Rhs, Output = Output>
        + Div<Rhs, Output = Output>
        + Rem<Rhs, Output = Output>
{
}
pub trait FixedPoint:
    Eq + PartialEq + Clone + Ord + PartialOrd + std::fmt::Debug + FixedPointOps + ToString
{
    fn fixed_zero() -> Self;
    fn fixed_is_zero(&self) -> bool;
    fn fixed_is_negative(&self) -> bool;
    fn fixed_sqr(&self) -> Self {
        self.clone() * self.clone()
    }
    fn fixed_sqrt(&self) -> Result<Self, Error>;
}
pub trait FixedPointInteger: FixedPoint {
    fn fixed_from_f64(value: f64, multiplier: &Self) -> Result<Self, Error>;
}
pub trait FixedPointDecimal: FixedPoint {
    fn fixed_from_f64(value: f64) -> Result<Self, Error>;
}
impl FixedPoint for BigInt {
    fn fixed_zero() -> Self {
        BigInt::default()
    }

    fn fixed_sqrt(&self) -> Result<Self, Error> {
        match self.sign() {
            Sign::Minus => {
                let e = Error::NegativeSqrtErr(self.to_string());
                error!("{:?}", e);
                return Err(e);
            }
            _ => Ok(BigInt::sqrt(&self)),
        }
    }

    fn fixed_is_zero(&self) -> bool {
        self.is_zero()
    }

    fn fixed_is_negative(&self) -> bool {
        self.is_negative()
    }
}
impl FixedPoint for Decimal {
    fn fixed_zero() -> Self {
        Decimal::ZERO
    }

    fn fixed_sqrt(&self) -> Result<Self, Error> {
        self.sqrt().ok_or_else(|| {
            let e = Error::NegativeSqrtErr(self.to_string());
            error!("{:?}", e);
            e
        })
    }

    fn fixed_is_zero(&self) -> bool {
        self.is_zero()
    }

    fn fixed_is_negative(&self) -> bool {
        self.is_sign_negative()
    }
}
impl FixedPoint for Ratio<BigInt> {
    fn fixed_zero() -> Self {
        Ratio::default()
    }

    fn fixed_sqrt(&self) -> Result<Self, Error> {
        let numer_tmp = self.denom() * self.numer();
        if numer_tmp.sign() == Sign::Minus {
            let e = Error::NegativeSqrtErr(self.to_string());
            error!("{:?}", e);
            return Err(e);
        }
        Ok(Ratio::new(numer_tmp.sqrt(), self.denom().to_owned()))
    }

    fn fixed_is_zero(&self) -> bool {
        self.is_zero()
    }

    fn fixed_is_negative(&self) -> bool {
        self.is_negative()
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
