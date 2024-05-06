use config::PoxConfig;
use types::{Error, FixedPoint, Pos2D};

pub trait PosTrait {
    type BaseType: FixedPoint;
    fn dist(&self, target: &Self) -> Result<Self::BaseType, Error>;
    fn dist_sqr(&self, target: &Self) -> Self::BaseType;
}
impl<T: FixedPoint> PosTrait for Pos2D<T> {
    type BaseType = T;
    fn dist(&self, target: &Self) -> Result<T, Error> {
        ((self.x.clone() - target.x.clone()).fixed_sqr()
            + (self.y.clone() - target.y.clone()).fixed_sqr())
        .fixed_sqrt()
    }
    fn dist_sqr(&self, target: &Self) -> T {
        (self.x.clone() - target.x.clone()).fixed_sqr()
            + (self.y.clone() - target.y.clone()).fixed_sqr()
    }
}
use std::fmt::Debug;
pub trait Kernel: Sized + Debug + std::marker::Sync {
    type BaseType: FixedPoint;
    type PosType: PosTrait<BaseType = Self::BaseType>;
    fn from_pox_cfg(config: &PoxConfig) -> Result<Self, Error>;
    fn denom(&self) -> Self::BaseType;
    fn eval_numer(&self, x1: &Self::PosType, x2: &Self::PosType) -> Self::BaseType;
}
#[cfg(test)]
mod tests {
    use crate::Quadratic;

    use super::*;
    use rust_decimal::{prelude::One, Decimal};
    use std::str::FromStr;

    #[test]
    fn test_pos_trait_dist() {
        let pos1 = Pos2D {
            x: Decimal::from_str("0.1").unwrap(),
            y: Decimal::from_str("0.2").unwrap(),
        };
        let pos2 = Pos2D {
            x: Decimal::from_str("0.3").unwrap(),
            y: Decimal::from_str("0.4").unwrap(),
        };
        assert_eq!(
            pos1.dist(&pos2).unwrap(),
            Decimal::from_str("0.2828427124746190097603377448").unwrap()
        );
    }
    #[test]
    fn test_pos_trait_dist_sqr() {
        let pos1 = Pos2D {
            x: Decimal::from_str("0.1").unwrap(),
            y: Decimal::from_str("0.2").unwrap(),
        };
        let pos2 = Pos2D {
            x: Decimal::from_str("0.3").unwrap(),
            y: Decimal::from_str("0.4").unwrap(),
        };
        assert_eq!(pos1.dist_sqr(&pos2), Decimal::from_str("0.08").unwrap());
    }
    #[test]
    fn test_kernel_eval() {
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
            max_dis_sqr: Decimal::from_str("0.01").unwrap(),
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
            Decimal::from_str("0.00").unwrap()
        );
        assert_eq!(kernel.denom(), Decimal::one());
    }
}
