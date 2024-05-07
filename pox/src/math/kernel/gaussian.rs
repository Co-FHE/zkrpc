use num_bigint::BigInt;
use num_rational::Ratio;
use rust_decimal::{
    prelude::{One, Zero},
    Decimal, MathematicalOps,
};
use rust_decimal_macros::dec;
use types::{Error, FixedPoint, FixedPointDecimal, FixedPointInteger, Pos2D};

use crate::{Kernel, PosTrait};

// trait GaussianImp{}
#[derive(Clone, Debug)]
struct GaussianVanilla {}
#[derive(Clone, Debug)]
pub struct GaussianTaylor {
    pub max_order: usize,
    pub sigma_range: Ratio<BigInt>,
}
// impl GaussianImp for GaussianTaylor{}
// impl GaussianImp for GaussianVanilla{}
// s^2 - x^2
#[derive(Clone, Debug)]
pub struct Quadratic<T: FixedPoint> {
    pub max_dis_sqr: T,
}
#[derive(Clone, Debug)]
pub struct Gaussian<T: FixedPoint, I> {
    pub sigma_sqr: T,
    pub implement_params: I,
}
#[derive(Clone, Debug)]
pub enum KernelKind<T: FixedPoint> {
    GaussianTaylor(Gaussian<T, GaussianTaylor>),
    Quadratic(Quadratic<T>),
}

impl Kernel for Quadratic<Decimal> {
    type BaseType = Decimal;
    type PosType = Pos2D<Decimal>;
    fn eval_numer(&self, x1: &Self::PosType, x2: &Self::PosType) -> Self::BaseType {
        let dis = x1.dist_sqr(x2);
        if dis > self.max_dis_sqr {
            Decimal::fixed_zero()
        } else {
            self.max_dis_sqr.clone() - dis
        }
    }

    fn denom(&self) -> Self::BaseType {
        Decimal::fixed_one()
    }

    fn from_pox_cfg(config: &config::PoxConfig) -> Result<Self, Error> {
        Ok(Self {
            max_dis_sqr: config.kernel.quadratic.max_dis_sqr.clone(),
        })
    }
}
impl Kernel for Quadratic<BigInt> {
    type BaseType = BigInt;

    type PosType = Pos2D<BigInt>;

    fn from_pox_cfg(config: &config::PoxConfig) -> Result<Self, Error> {
        Ok(Self {
            max_dis_sqr: BigInt::fixed_from_decimal(
                config.kernel.quadratic.max_dis_sqr.clone(),
                config.coordinate_precision_bigint * 2,
            )?,
        })
    }

    fn denom(&self) -> Self::BaseType {
        BigInt::fixed_one()
    }

    fn eval_numer(&self, x1: &Self::PosType, x2: &Self::PosType) -> Self::BaseType {
        let dis = x1.dist_sqr(x2);
        if dis > self.max_dis_sqr {
            BigInt::fixed_zero()
        } else {
            self.max_dis_sqr.clone() - dis
        }
    }
}
impl Kernel for KernelKind<BigInt> {
    type BaseType = BigInt;

    type PosType = Pos2D<BigInt>;

    fn from_pox_cfg(config: &config::PoxConfig) -> Result<Self, Error> {
        match config.kernel.kernel_type {
            config::KernelTypeConfig::GaussianTaylor => {
                Ok(Self::GaussianTaylor(Gaussian::from_pox_cfg(config)?))
            }
            config::KernelTypeConfig::Quadratic => {
                Ok(Self::Quadratic(Quadratic::from_pox_cfg(config)?))
            }
        }
    }

    fn denom(&self) -> Self::BaseType {
        match self {
            Self::GaussianTaylor(kernel) => kernel.denom(),
            Self::Quadratic(kernel) => kernel.denom(),
        }
    }

    fn eval_numer(&self, x1: &Self::PosType, x2: &Self::PosType) -> Self::BaseType {
        match self {
            Self::GaussianTaylor(kernel) => kernel.eval_numer(x1, x2),
            Self::Quadratic(kernel) => kernel.eval_numer(x1, x2),
        }
    }
}

impl Kernel for Gaussian<Decimal, GaussianVanilla> {
    type BaseType = Decimal;
    type PosType = Pos2D<Decimal>;
    fn eval_numer(&self, x1: &Self::PosType, x2: &Self::PosType) -> Self::BaseType {
        let exp = -x1.dist_sqr(x2) / Decimal::TWO / self.sigma_sqr;
        let exp = exp.exp();
        exp
    }

    fn denom(&self) -> Self::BaseType {
        Decimal::PI * self.sigma_sqr
    }

    fn from_pox_cfg(config: &config::PoxConfig) -> Result<Self, Error> {
        Ok(Self {
            sigma_sqr: config.kernel.gaussian.sigma.clone() * config.kernel.gaussian.sigma.clone(),
            implement_params: GaussianVanilla {},
        })
    }
}
//\Sum{(-1/2)^k * x^{2k} / k!}
// O(x^{2k}) = x^{2k}/{k! * 2^k}
// x^{2k}/{k! * 2^k} < \epsilon
// x^{2k} < \epsilon * k! * 2^k
// log(x) * 2k < log(\epsilon) + log(k!) + log(2) * k
// k! ~= sqrt(2 * pi * k) * (k / e) ^ k
// log(x) * 2k < log(\epsilon) + 0.5*(log(2)+log(pi)+log(k)) + k * (log(k)-1)
use rust_decimal::prelude::*;
#[allow(dead_code)]
fn torlerence_epsilon(x: Decimal, epsilon: Decimal) -> usize {
    let mut k = dec![1];
    let log_x = x.ln();
    let log_epsilon = epsilon.ln();
    loop {
        let log_k = Decimal::from(k).ln();
        let sum = log_x * dec![2] * k
            - log_epsilon
            - k * dec!(2).ln()
            - dec![0.5] * (Decimal::from(2).ln() + Decimal::PI.ln() + log_k)
            - k * (log_k - Decimal::one());
        if sum < Decimal::zero() {
            break;
        }
        k += Decimal::one();
    }
    k.to_usize().unwrap()
}

//  1 - x^2/2 + x^4/8 - x^6/48 + x^8/384 - x^10/3840
// denom: m!*2^m*sigma^(2m)
// let b = 1
// b*=(i+1)*2*sigma^2
// num = b*x^(2i)
fn taylor_exp_numer(x_sqr: BigInt, sigma_sqr: BigInt, max_order: usize) -> BigInt {
    let mut numer: Vec<BigInt> = vec![BigInt::one(); max_order + 1];
    for i in 1..=max_order {
        numer[i] = -numer[i - 1].clone() * x_sqr.clone();
    }
    let mut b = BigInt::one();
    for i in (0..max_order).rev() {
        b *= (i + 1) * 2 * sigma_sqr.clone();
        numer[i] *= b.clone();
    }
    numer.iter().sum()
}
fn taylor_exp_denom(sigma_sqr: BigInt, max_order: usize) -> BigInt {
    factorial(max_order) * BigInt::from(2).pow(max_order as u32) * sigma_sqr.pow(max_order as u32)
}
impl Kernel for Gaussian<BigInt, GaussianTaylor> {
    type BaseType = BigInt;
    type PosType = Pos2D<BigInt>;
    fn eval_numer(&self, x1: &Self::PosType, x2: &Self::PosType) -> Self::BaseType {
        let x_sqr = x1.dist_sqr(x2);
        if Ratio::<BigInt>::new(x_sqr.clone(), self.sigma_sqr.clone())
            > self.implement_params.sigma_range.clone() * self.implement_params.sigma_range.clone()
        {
            return BigInt::zero();
        }
        let numer = taylor_exp_numer(
            x_sqr,
            self.sigma_sqr.clone(),
            self.implement_params.max_order,
        );
        if numer.is_negative() {
            return BigInt::zero();
        }
        numer
    }
    fn denom(&self) -> Self::BaseType {
        taylor_exp_denom(self.sigma_sqr.clone(), self.implement_params.max_order)
    }

    fn from_pox_cfg(config: &config::PoxConfig) -> Result<Self, Error> {
        let sigma = BigInt::fixed_from_decimal(
            config.kernel.gaussian.sigma.clone(),
            config.coordinate_precision_bigint,
        )?;
        Ok(Self {
            sigma_sqr: sigma.clone() * sigma,
            implement_params: GaussianTaylor {
                max_order: config.kernel.gaussian.taylor.max_order,
                sigma_range: Ratio::<BigInt>::fixed_from_decimal(
                    &config.kernel.gaussian.taylor.sigma_range,
                )?,
            },
        })
    }
}

fn factorial(i: usize) -> BigInt {
    (1..=i)
        .map(|x| BigInt::from(x))
        .fold(BigInt::one(), |acc, x| acc * x)
}
// impl<T: FixedPoint, I> Gaussian<T, I> {
//     pub fn new(sigma: T, param: I) -> Result<Self, Error> {
//         if sigma.fixed_is_zero() || sigma.fixed_is_negative() {
//             return Err(Error::SigmaZeroOrNegative(sigma.to_string()));
//         }
//         Ok(Self {
//             sigma_sqr: sigma.clone() * sigma.clone(),
//             implement_params: param,
//         })
//     }
// }
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    #[test]
    fn test_bear_epsilon() {
        assert_eq!(
            torlerence_epsilon(
                Decimal::from_str("3").unwrap(),
                Decimal::from_str("0.00001").unwrap(),
            ),
            20
        );
    }
    #[test]
    fn test_factorial() {
        assert_eq!(factorial(5), BigInt::from_str("120").unwrap());
        let n = taylor_exp_numer(BigInt::from(1), BigInt::from(1), 50);
        let d = taylor_exp_denom(BigInt::from(1), 50);
        assert_eq!(
            n,
            BigInt::from_str(
                "20769565669502534364226566022587687079394874821162568392644254779986813920873701"
            )
            .unwrap()
        );
        assert_eq!(
            d,
            BigInt::from_str(
                "34243224702511976248246432895208185975118675053719198827915654463488000000000000"
            )
            .unwrap()
        );
        // ClearAll["Global`*"]
        // f[x_] := Exp[-(x/s)^2/2];
        // n = 100;
        // Normal[Series[f[x], {x, 0, n}]]
        let n = taylor_exp_numer(BigInt::from(11 * 11), BigInt::from(19 * 19), 50);
        let d = taylor_exp_denom(BigInt::from(19 * 19), 50);
        let r = Ratio::<BigInt>::new(n, d);
        //148450131305133953855417326680832070182161192942292012136875524213320647992843081334565322435885022157518844587897237180770784378928132881325235867548709821591615063710072197642142655675488612154690218661
        assert_eq!(*r.numer(), BigInt::from_str("148450131305133953855417326680832070182161192942292012136875524213320647992843081334565322435885022157518844587897237180770784378928132881325235867548709821591615063710072197642142655675488612154690218661").unwrap());
        // 175535115887523253049417960891066168877319065015421838450740934347255819063392392031587462517650551642846938439275426966830608311436767833614778053699465617757764535099966691312012288713555968000000000000
        assert_eq!(*r.denom(), BigInt::from_str("175535115887523253049417960891066168877319065015421838450740934347255819063392392031587462517650551642846938439275426966830608311436767833614778053699465617757764535099966691312012288713555968000000000000").unwrap());
    }
    #[test]
    fn test_quadratic_eval() {
        let kernel = Quadratic {
            max_dis_sqr: Decimal::from_str("0.1").unwrap(),
        };
        let pos1 = Pos2D {
            x: Decimal::from_str("0.1").unwrap(),
            y: Decimal::from_str("0.2").unwrap(),
        };
        let pos2 = Pos2D {
            x: Decimal::from_str("0.3").unwrap(),
            y: Decimal::from_str("0.4").unwrap(),
        };
        assert_eq!(
            kernel.eval_numer(&pos1, &pos2),
            Decimal::from_str("0.02").unwrap()
        );
        assert_eq!(kernel.denom(), Decimal::one());
        let kernel = Quadratic {
            max_dis_sqr: BigInt::from(100000),
        };
        let pos1 = Pos2D {
            x: BigInt::from(100),
            y: BigInt::from(200),
        };
        let pos2 = Pos2D {
            x: BigInt::from(300),
            y: BigInt::from(400),
        };
        assert_eq!(kernel.eval_numer(&pos1, &pos2), BigInt::from(20000));
        let kernel = Quadratic {
            max_dis_sqr: BigInt::from(10000),
        };
        assert_eq!(kernel.eval_numer(&pos1, &pos2), BigInt::from(0));
    }
    #[test]
    fn test_gaussian_vanilla_eval() {
        let kernel = Gaussian {
            sigma_sqr: Decimal::from_str("0.01").unwrap(),
            implement_params: GaussianVanilla {},
        };
        let pos1 = Pos2D {
            x: Decimal::from_str("0.1").unwrap(),
            y: Decimal::from_str("0.2").unwrap(),
        };
        let pos2 = Pos2D {
            x: Decimal::from_str("0.3").unwrap(),
            y: Decimal::from_str("0.4").unwrap(),
        };
        assert_eq!(
            kernel.eval_numer(&pos1, &pos2),
            Decimal::from_str("0.0183156388950786941119281022").unwrap(),
        );
        assert_eq!(
            kernel.denom(),
            Decimal::PI * Decimal::from_str("0.01").unwrap()
        );
    }
    #[test]
    fn test_gaussian_taylor_eval() {
        let kernel = Gaussian {
            sigma_sqr: BigInt::from(16),
            implement_params: GaussianTaylor {
                max_order: 5,
                sigma_range: Ratio::<BigInt>::from_str("3").unwrap(),
            },
        };
        let pos1 = Pos2D {
            x: BigInt::from(1),
            y: BigInt::from(2),
        };
        let pos2 = Pos2D {
            x: BigInt::from(2),
            y: BigInt::from(2),
        };
        assert_eq!(
            kernel.eval_numer(&pos1, &pos2),
            BigInt::from_str("3902648479").unwrap()
        );
        assert_eq!(kernel.denom(), BigInt::from_str("4026531840").unwrap());
    }
}
