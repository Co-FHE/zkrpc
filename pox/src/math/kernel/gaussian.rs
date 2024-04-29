use std::ops::{Add, Div, Mul, Sub};

use num_bigint::BigInt;
use rust_decimal::{prelude::Zero, Decimal, MathematicalOps};
use types::{Error, FixedPoint, FixedPointOps, Pos2D, Terminal};

use crate::{Kernel, PosTrait};

// trait GaussianImp{}
struct GaussianVanilla<const CALC_COEF: bool> {}
struct GaussianTylor {
    pub max_order: usize,
    pub sigma_range: Decimal,
}
// impl GaussianImp for GaussianTylor{}
// impl GaussianImp for GaussianVanilla{}
// s^2 - x^2
pub struct Quadratic<T: FixedPoint> {
    pub max_dis_sqr: T,
}
struct Gaussian<T, I> {
    sigma: T,
    sigma_sqr: T,
    implement_param: I,
}

impl<P, T> Kernel<P, T> for Quadratic<T>
where
    P: PosTrait<T>,
    T: FixedPoint,
{
    fn eval(&self, x1: &P, x2: &P) -> T {
        let dis = x1.dist_sqr(x2);
        if dis > self.max_dis_sqr {
            T::fixed_zero()
        } else {
            self.max_dis_sqr.clone() - dis
        }
    }
}

impl<P, const CALC_COEF: bool> Kernel<P, Decimal> for Gaussian<Decimal, GaussianVanilla<CALC_COEF>>
where
    P: PosTrait<Decimal>,
{
    fn eval(&self, x1: &P, x2: &P) -> Decimal {
        let mut sum = Decimal::zero();
        let exp = -x1.dist_sqr(x2) / Decimal::TWO / self.sigma.fixed_sqr();
        let exp = exp.exp();
        if CALC_COEF {
            sum = exp;
        } else {
            sum = exp / Decimal::PI / self.sigma_sqr;
        }
        sum
    }
}
//\Sum{(-1/2)^k * x^{2k} / k!}
//  1 - x^2/2 + x^4/8 - x^6/48 + x^8/384 - x^10/3840
impl<P> Kernel<P, BigInt> for Gaussian<BigInt, GaussianTylor>
where
    P: PosTrait<BigInt>,
{
    fn eval(&self, x1: &P, x2: &P) -> BigInt {
        todo!()
    }
}
impl<T: FixedPoint, I> Gaussian<T, I> {
    pub fn new(sigma: T, param: I) -> Result<Self, Error> {
        if sigma.fixed_is_zero() || sigma.fixed_is_negative() {
            return Err(Error::SigmaZeroOrNegative(sigma.to_string()));
        }
        Ok(Self {
            sigma: sigma.clone(),
            sigma_sqr: sigma.clone() * sigma.clone(),
            implement_param: param,
        })
    }
}
