use std::ops::{Add, Div, Mul, Sub};

use num_bigint::BigInt;
use rust_decimal::{prelude::Zero, Decimal, MathematicalOps};
use types::{Error, FixedPoint, FixedPointOps, Pos2D, Terminal};

use crate::{Kernel, PosTrait};

// trait GaussianImp{}
#[derive(Clone, Debug)]
struct GaussianVanilla<const CALC_COEF: bool> {}
struct GaussianTylor {
    pub max_order: usize,
    pub sigma_range: Decimal,
}
// impl GaussianImp for GaussianTylor{}
// impl GaussianImp for GaussianVanilla{}
// s^2 - x^2
#[derive(Clone, Debug)]
pub struct Quadratic<T: FixedPoint> {
    pub max_dis_sqr: T,
}
#[derive(Clone, Debug)]
struct Gaussian<T, I> {
    sigma: T,
    sigma_sqr: T,
    implement_param: I,
}

impl<T: FixedPoint> Kernel for Quadratic<T> {
    type BaseType = T;
    type PosType = Pos2D<T>;
    fn eval(&self, x1: &Pos2D<T>, x2: &Pos2D<T>) -> T {
        let dis = x1.dist_sqr(x2);
        if dis > self.max_dis_sqr {
            T::fixed_zero()
        } else {
            self.max_dis_sqr.clone() - dis
        }
    }
}

impl<const CALC_COEF: bool> Kernel for Gaussian<Decimal, GaussianVanilla<CALC_COEF>> {
    type BaseType = Decimal;
    type PosType = Pos2D<Decimal>;
    fn eval(&self, x1: &Pos2D<Decimal>, x2: &Pos2D<Decimal>) -> Decimal {
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
impl Kernel for Gaussian<BigInt, GaussianTylor> {
    type BaseType = BigInt;
    type PosType = Pos2D<BigInt>;
    fn eval(&self, x1: &Self::PosType, x2: &Self::PosType) -> BigInt {
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
