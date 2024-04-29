use halo2curves::Coordinates;
use num_bigint::BigInt;
use rust_decimal::Decimal;

use crate::{Error, FixedPoint, FixedPointDecimal, FixedPointInteger, Satellite, Terminal};
#[derive(Debug, Clone)]
pub struct Pos2D<T: FixedPoint> {
    pub x: T,
    pub y: T,
}
#[derive(Debug, Clone)]
pub struct Pos3D<T: FixedPoint> {
    pub x: T,
    pub y: T,
    pub height: T,
}
impl Pos3D<Decimal> {
    pub fn new_from_f64(x: f64, y: f64, height: f64) -> Result<Self, Error> {
        Ok(Self {
            x: Decimal::fixed_from_f64(x)?,
            y: Decimal::fixed_from_f64(y)?,
            height: Decimal::fixed_from_f64(height)?,
        })
    }
}
impl<T: FixedPoint> Pos2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
    pub fn x(&self) -> T {
        self.x.clone()
    }
    pub fn y(&self) -> T {
        self.y.clone()
    }
}
impl<T: FixedPointDecimal> Pos2D<T> {
    pub fn new_from_flat_point_f64(point: flat_projection::FlatPoint<f64>) -> Result<Self, Error> {
        Ok(Self {
            x: T::fixed_from_f64(point.x)?,
            y: T::fixed_from_f64(point.y)?,
        })
    }
}
impl<T: FixedPoint> Pos3D<T> {
    pub fn new(x: T, y: T, height: T) -> Self {
        Self { x, y, height }
    }
    pub fn x(&self) -> T {
        self.x.clone()
    }
    pub fn y(&self) -> T {
        self.y.clone()
    }
    pub fn height(&self) -> T {
        self.height.clone()
    }
}
impl Pos2D<Decimal> {
    pub fn new_from_f64(x: f64, y: f64) -> Result<Self, Error> {
        Ok(Self {
            x: Decimal::fixed_from_f64(x)?,
            y: Decimal::fixed_from_f64(y)?,
        })
    }
}

impl Pos2D<BigInt> {
    pub fn new_from_f64(x: f64, y: f64, coordinate_multiplier: &BigInt) -> Result<Self, Error> {
        Ok(Self {
            x: BigInt::fixed_from_f64(x, coordinate_multiplier)?,
            y: BigInt::fixed_from_f64(x, coordinate_multiplier)?,
        })
    }
    pub fn new_from_decimal(x: Decimal, y: Decimal, exp: u32) -> Result<Self, Error> {
        Ok(Self {
            x: BigInt::fixed_from_decimal(x, exp)?,
            y: BigInt::fixed_from_decimal(y, exp)?,
        })
    }
}
impl Pos3D<BigInt> {
    pub fn new_from_f64(x: f64, y: f64, coordinate_multiplier: &BigInt) -> Result<Self, Error> {
        Ok(Self {
            x: BigInt::fixed_from_f64(x, coordinate_multiplier)?,
            y: BigInt::fixed_from_f64(x, coordinate_multiplier)?,
            height: BigInt::fixed_from_f64(x, coordinate_multiplier)?,
        })
    }
    pub fn new_from_decimal(
        x: Decimal,
        y: Decimal,
        height: Decimal,
        exp: u32,
    ) -> Result<Self, Error> {
        Ok(Self {
            x: BigInt::fixed_from_decimal(x, exp)?,
            y: BigInt::fixed_from_decimal(y, exp)?,
            height: BigInt::fixed_from_decimal(height, exp)?,
        })
    }
}

pub trait GetPos2D<T: FixedPoint> {
    fn get_pos_2d(&self) -> Pos2D<T>;
}
impl<T: FixedPoint> GetPos2D<T> for Terminal<T> {
    fn get_pos_2d(&self) -> Pos2D<T> {
        self.position.clone()
    }
}

impl<T: FixedPoint> GetPos2D<T> for Satellite<T> {
    fn get_pos_2d(&self) -> Pos2D<T> {
        Pos2D {
            x: self.position.x.clone(),
            y: self.position.y.clone(),
        }
    }
}
