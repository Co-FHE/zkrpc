use config::PoxConfig;
// use config::config::{COORDINATE_PRECISION_BIGINT, RSPR_PRECISION_BIGINT};
use lazy_static::lazy_static;
use num_bigint::{BigInt, BigUint, ToBigInt};
use rust_decimal::Decimal;

use crate::{
    EndPointFrom, Error, FixedPoint, FixedPointDecimal, FixedPointInteger, Packets, Pos2D,
};

// lazy_static! {
//     static ref COORDINATE_MULTIPLIER_BIGINT: BigInt = BigUint::from(10u32)
//         .pow(COORDINATE_PRECISION_BIGINT)
//         .to_bigint()
//         .unwrap();
//     static ref RSPR_MULTIPLIER_BIGINT: BigInt = BigUint::from(10u32)
//         .pow(RSPR_PRECISION_BIGINT)
//         .to_bigint()
//         .unwrap();
// }
#[derive(Debug, Clone)]
pub struct Terminal<T: FixedPoint> {
    pub address: String,
    pub position: Pos2D<T>,
    pub alpha: Alpha<T>,
    // terminal may do not receive packets
    pub terminal_packets: Option<Packets>,
}
#[derive(Debug, Clone)]
pub struct Alpha<T: FixedPoint> {
    pub rspr: T,
}
impl<T: FixedPoint> Alpha<T> {
    pub fn new(rspr: T) -> Self {
        Self { rspr }
    }
}
impl<T: FixedPoint> Terminal<T> {
    pub fn new(address: String, x: T, y: T, alpha: Alpha<T>, packets: Option<Packets>) -> Self {
        Self {
            address,
            position: Pos2D::<T>::new(x, y),
            alpha,
            terminal_packets: packets,
        }
    }
}

impl Alpha<BigInt> {
    pub fn new_from_decimal(rspr: Decimal, exp: u32) -> Result<Self, Error> {
        Ok(Self {
            rspr: BigInt::fixed_from_decimal(rspr, exp)?,
        })
    }
}
impl Terminal<BigInt> {
    pub fn new_from_decimal(
        address: String,
        x: Decimal,
        y: Decimal,
        alpha: Decimal,
        coor_exp: u32,
        rspr_exp: u32,
        packets: Option<Packets>,
    ) -> Result<Self, Error> {
        Ok(Self {
            address,
            position: Pos2D::<BigInt>::new_from_decimal(x, y, coor_exp)?,
            alpha: Alpha::<BigInt>::new_from_decimal(alpha, rspr_exp)?,
            terminal_packets: packets,
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
    pub fn new_from_f64(
        address: String,
        x: f64,
        y: f64,
        alpha: f64,
        packets: Option<Packets>,
    ) -> Result<Self, Error> {
        Ok(Self {
            address,
            position: Pos2D::<Decimal>::new_from_f64(x, y)?,
            alpha: Alpha::<Decimal>::new_from_f64(alpha)?,
            terminal_packets: packets,
        })
    }
}
impl EndPointFrom<Terminal<Decimal>> for Terminal<BigInt> {
    fn from_with_config(value: Terminal<Decimal>, cfg: &PoxConfig) -> Result<Self, Error> {
        Ok(Self {
            address: value.address,
            position: Pos2D::<BigInt>::new_from_decimal(
                value.position.x,
                value.position.y,
                cfg.cooridnate_precision_bigint,
            )?,
            alpha: Alpha::<BigInt> {
                rspr: BigInt::fixed_from_decimal(value.alpha.rspr, cfg.rspr_precision_bigint)?,
            },
            terminal_packets: value.terminal_packets,
        })
    }
}
