use std::{
    ops::{Add, Div, Mul, Rem, Sub},
    str::FromStr,
};

use halo2curves::pasta::Fp;
use num_bigint::{BigInt, Sign};
use num_rational::{BigRational, Ratio};
use num_traits::*;
use rust_decimal::{prelude::FromPrimitive, Decimal, MathematicalOps};
use tracing::error;

use crate::Error;
use halo2curves::ff::Field;

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
    fn fixed_one() -> Self;
    fn fixed_zero() -> Self;
    fn fixed_is_zero(&self) -> bool;
    fn fixed_is_negative(&self) -> bool;
    fn fixed_sqr(&self) -> Self {
        self.clone() * self.clone()
    }
    fn fixed_sqrt(&self) -> Result<Self, Error>;
}
pub trait FixedPointInteger: FixedPoint {
    fn to_fp(&self) -> Result<Fp, Error>;
    fn fixed_magnitude_to_u64(&self) -> Result<u64, Error>;
    fn fixed_log_magnitude_to_u64(&self) -> Result<u64, Error>;
    fn fixed_to_decimal(&self, exp: u32) -> Result<Decimal, Error>;
    fn fixed_from_f64(value: f64, multiplier: &Self) -> Result<Self, Error>;
    fn fixed_from_decimal(value: Decimal, exp: u32) -> Result<Self, Error>;
}
pub trait FixedPointDecimal: FixedPoint {
    fn fixed_from_f64(value: f64) -> Result<Self, Error>;
    fn fixed_to_f64(&self) -> Result<f64, Error>;
    fn fixed_from_decimal(value: &Decimal) -> Result<Self, Error>;
    fn fixed_to_decimal(&self) -> Result<Decimal, Error>;
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

    fn fixed_sqr(&self) -> Self {
        self.clone() * self.clone()
    }

    fn fixed_one() -> Self {
        BigInt::one()
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

    fn fixed_one() -> Self {
        Decimal::ONE
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

    fn fixed_one() -> Self {
        Ratio::one()
    }
}

impl FixedPointInteger for BigInt {
    fn fixed_from_f64(value: f64, multiplier: &Self) -> Result<Self, Error> {
        let r = BigRational::from_float(value).ok_or({
            let e = Error::BigIntConversionErr(
                format!("value: {}, multiplier: {}", value.to_string(), multiplier),
                "from f64".to_owned(),
            );
            error!("{:?}", e);
            e
        })? * multiplier;
        return Ok(r.to_integer());
    }
    fn fixed_magnitude_to_u64(&self) -> Result<u64, Error> {
        self.magnitude().to_u64().ok_or_else(|| {
            let e = Error::BigIntConversionErr(self.to_string(), "to u64".to_owned());
            error!("{:?}", e);
            e
        })
    }
    fn fixed_log_magnitude_to_u64(&self) -> Result<u64, Error> {
        Ok(self.magnitude().to_string().len() as u64)
    }
    fn to_fp(&self) -> Result<Fp, Error> {
        let (sig, mut bytes) = self.to_u64_digits();
        if bytes.len() == 0 {
            return Ok(Fp::zero());
        }
        bytes.resize(((bytes.len() - 1) / 4 + 1) * 4, 0);
        let mut result = Fp::zero();
        for i in 0..bytes.len() / 4 {
            let fp = Fp::from_raw([
                bytes[i * 4],
                bytes[i * 4 + 1],
                bytes[i * 4 + 2],
                bytes[i * 4 + 3],
            ]);
            result += fp * Fp::from(2).pow(vec![64 * 4 * i as u64]);
        }
        if sig == Sign::Minus {
            Ok(-result)
        } else {
            Ok(result)
        }
    }
    // limited by the size of decimal
    fn fixed_from_decimal(value: Decimal, exp: u32) -> Result<Self, Error> {
        // TODO: use shift_left instead of powi
        let multiplier = Decimal::from(10).checked_powu(exp as u64);
        let multiplied_decimal = value
            * multiplier.ok_or_else(|| {
                let e = Error::DecimalErr(value, exp);
                error!("{}", e.to_string());
                e
            })?;
        let as_bigint = BigInt::from_str(&multiplied_decimal.trunc().to_string()).map_err(|e| {
            let e =
                Error::BigIntConversionErr(multiplied_decimal.trunc().to_string(), e.to_string());
            error!("{:?}", e);
            e
        })?;
        Ok(as_bigint)
    }

    fn fixed_to_decimal(&self, exp: u32) -> Result<Decimal, Error> {
        let multiplier = Decimal::from(10).checked_powi(exp as i64).ok_or_else(|| {
            let e = Error::DecimalErr(Decimal::from(10), exp);
            error!("{}", e.to_string());
            e
        })?;
        let as_decimal = Decimal::from_str(&self.to_string()).map_err(|e| {
            let e = Error::BigIntConversionErr(self.to_string(), e.to_string());
            error!("{:?}", e);
            e
        })?;
        Ok(as_decimal / multiplier)
    }
}

impl FixedPointDecimal for Decimal {
    fn fixed_from_f64(value: f64) -> Result<Self, Error> {
        Ok(Decimal::from_f64(value).ok_or_else(|| {
            let e = Error::DecimalParseErr(value.to_string(), "from f64".to_owned());
            error!("{:?}", e);
            e
        })?)
    }

    fn fixed_to_f64(&self) -> Result<f64, Error> {
        Ok(self.to_f64().ok_or_else(|| {
            let e = Error::DecimalParseErr(self.to_string(), "to f64".to_owned());
            error!("{:?}", e);
            e
        })?)
    }

    fn fixed_from_decimal(value: &Decimal) -> Result<Self, Error> {
        Ok(value.clone())
    }

    fn fixed_to_decimal(&self) -> Result<Decimal, Error> {
        Ok(self.clone())
    }
}

impl FixedPointDecimal for Ratio<BigInt> {
    fn fixed_from_f64(value: f64) -> Result<Self, Error> {
        let r = BigRational::from_float(value).ok_or_else(|| {
            let e = Error::BigRationalConversionErr(value.to_string(), "from f64".to_owned());
            error!("{:?}", e);
            e
        })?;
        Ok(r)
    }

    fn fixed_to_f64(&self) -> Result<f64, Error> {
        Ok(self.to_f64().ok_or_else(|| {
            let e = Error::BigRationalConversionErr(self.to_string(), "to f64".to_owned());
            error!("{:?}", e);
            e
        })?)
    }

    fn fixed_from_decimal(value: &Decimal) -> Result<Self, Error> {
        let as_str = format!("{}", value);
        let parts: Vec<&str> = as_str.split('.').collect();

        let (integer_part, decimal_part) = match parts.len() {
            1 => (parts[0], ""),
            2 => (parts[0], parts[1]),
            _ => return Err(Error::DecimalErr(value.clone(), 0)),
        };

        let decimal_places = decimal_part.len() as u32;

        let numerator =
            BigInt::from_str(&(integer_part.to_string() + decimal_part)).map_err(|e| {
                let e = Error::BigIntConversionErr(value.to_string(), e.to_string());
                error!("{:?}", e);
                e
            })?;
        let denominator = BigInt::from(10).pow(decimal_places);
        if BigRational::new(numerator.clone(), denominator.clone()).fixed_to_decimal()?
            != value.clone()
        {
            return Err(Error::BigRationalConversionErr(
                format!(
                    "{}/{}, indead value {}",
                    numerator.to_string(),
                    denominator.to_string(),
                    value
                ),
                "from decimal".to_owned(),
            ));
        }
        return Ok(BigRational::new(numerator, denominator));
    }

    fn fixed_to_decimal(&self) -> Result<Decimal, Error> {
        let number = self.numer().to_string().parse::<Decimal>().map_err(|e| {
            let e = Error::DecimalParseErr(self.numer().to_string(), e.to_string());
            error!("{:?}", e);
            e
        })?;
        let denom = self.denom().to_string().parse::<Decimal>().map_err(|e| {
            let e = Error::DecimalParseErr(self.denom().to_string(), e.to_string());
            error!("{:?}", e);
            e
        })?;
        Ok(number / denom)
    }
}
// TODO test
#[cfg(test)]
mod tests {
    use halo2curves::ff::Field;
    use rust_decimal_macros::dec;

    use super::*;
    use crate::Error;

    #[test]
    fn test_fixed_point_integer() {
        let value = Decimal::from_f64(-1.0).unwrap();
        let exp = 18;
        let fixed = BigInt::fixed_from_decimal(value, exp).unwrap();
        let fixed2 = BigInt::fixed_from_f64(-1.0, &BigInt::from(10).pow(exp)).unwrap();
        assert_eq!(fixed, fixed2);
        assert_eq!(fixed.fixed_is_negative(), true);
        assert_eq!(fixed.fixed_is_zero(), false);
        assert_eq!(
            fixed.fixed_sqr(),
            BigInt::from(1) * BigInt::from(10).pow(exp * 2)
        );
        assert_eq!(
            fixed.fixed_magnitude_to_u64().unwrap(),
            1_000_000_000_000_000_000_u64
        );
        assert_eq!(fixed.fixed_log_magnitude_to_u64().unwrap(), 19);
        let decimal = fixed.fixed_to_decimal(exp).unwrap();
        assert_eq!(value, decimal);

        let value = Decimal::from_f64(-1.0).unwrap();
        let exp = 180;
        assert_eq!(
            BigInt::fixed_from_decimal(value, exp),
            Err(Error::DecimalErr(value, exp))
        );
        let fixed = BigInt::from(-1) * BigInt::from(10).pow(exp);
        assert_eq!(
            fixed.fixed_sqr(),
            BigInt::from(1) * BigInt::from(10).pow(exp * 2)
        );
        assert_eq!(fixed.fixed_sqr().fixed_sqrt().unwrap(), -fixed);
    }

    #[test]
    fn test_fixed_point_decimal() {
        let value = -3.2395;
        let fixed = Decimal::fixed_from_f64(value).unwrap();
        let fixed2 = Decimal::fixed_from_f64(value).unwrap();
        assert_eq!(fixed, fixed2);
        let f = fixed.fixed_to_f64().unwrap();
        assert_eq!(value, f);
        assert_eq!(fixed2.fixed_sqr().fixed_sqrt().unwrap(), -fixed2);
        assert!(fixed2.fixed_is_negative());
        assert!(!fixed2.fixed_is_zero());
    }
    #[test]
    fn test_fixed_point_ratio() {
        let value = -3.2395;
        let fixed = Ratio::<BigInt>::fixed_from_f64(value).unwrap();
        let fixed2 = Ratio::<BigInt>::fixed_from_f64(value).unwrap();
        assert_eq!(fixed, fixed2);
        let f = fixed.fixed_to_f64().unwrap();
        assert_eq!(value, f);
        assert_eq!(
            fixed2.clone().fixed_sqr().fixed_sqrt().unwrap(),
            -fixed2.clone()
        );
        assert!(fixed2.fixed_is_negative());
        assert!(!fixed2.fixed_is_zero());

        let fixed = Ratio::<BigInt>::from_str(
            "-1239861872469812687123981729487123124/91872398164871268975618925610273481235",
        )
        .unwrap();
        let fixed2 = fixed.clone();
        assert_eq!(fixed, fixed2);
        assert_eq!(
            fixed2.clone().fixed_sqr().fixed_sqrt().unwrap(),
            -fixed2.clone()
        );
        assert!(fixed2.fixed_is_negative());
        assert!(!fixed2.fixed_is_zero());
        let tests = vec![
            dec![123.123],
            dec![1.],
            dec![0.],
            dec![12343412.1231244],
            dec![5],
        ];
        for d in tests {
            let fixed = Ratio::<BigInt>::fixed_from_decimal(&d)
                .unwrap()
                .fixed_to_decimal()
                .unwrap();
            assert_eq!(d, fixed);
        }
    }
    #[test]
    fn test_fp() {
        assert_eq!(Fp::one(), BigInt::from(1).to_fp().unwrap());
        assert_eq!(Fp::zero(), BigInt::from(0).to_fp().unwrap());
        assert_eq!(Fp::from(25), BigInt::from(25).to_fp().unwrap());
        assert_eq!(
            -Fp::from(52914).pow(vec![30000]),
            -BigInt::from(52914).pow(30000_u64).to_fp().unwrap()
        );
    }
}
