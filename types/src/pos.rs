use num_bigint::BigInt;
use rust_decimal::Decimal;

use crate::{Error, FixedPoint, FixedPointDecimal, FixedPointInteger, Remote, Terminal};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pos2D<T: FixedPoint> {
    pub x: T,
    pub y: T,
}
#[derive(Debug, Clone, PartialEq, Eq)]
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
            y: BigInt::fixed_from_f64(y, coordinate_multiplier)?,
        })
    }
    pub fn new_from_decimal(x: Decimal, y: Decimal, exp: u32) -> Result<Self, Error> {
        Ok(Self {
            x: BigInt::fixed_from_decimal(x, exp)?,
            y: BigInt::fixed_from_decimal(y, exp)?,
        })
    }
    pub fn to_decimal(&self, exp: u32) -> Result<Pos2D<Decimal>, Error> {
        Ok(Pos2D {
            x: self.x.fixed_to_decimal(exp)?,
            y: self.y.fixed_to_decimal(exp)?,
        })
    }
}
impl Pos3D<BigInt> {
    pub fn new_from_f64(
        x: f64,
        y: f64,
        height: f64,
        coordinate_multiplier: &BigInt,
    ) -> Result<Self, Error> {
        Ok(Self {
            x: BigInt::fixed_from_f64(x, coordinate_multiplier)?,
            y: BigInt::fixed_from_f64(y, coordinate_multiplier)?,
            height: BigInt::fixed_from_f64(height, coordinate_multiplier)?,
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
    pub fn to_decimal(&self, exp: u32) -> Result<Pos3D<Decimal>, Error> {
        Ok(Pos3D {
            x: self.x.fixed_to_decimal(exp)?,
            y: self.y.fixed_to_decimal(exp)?,
            height: self.height.fixed_to_decimal(exp)?,
        })
    }
}

pub trait GetPos2D {
    type BaseType: FixedPoint;
    fn get_pos_2d(&self) -> Pos2D<Self::BaseType>;
}
impl<T: FixedPoint> GetPos2D for Terminal<T> {
    type BaseType = T;
    fn get_pos_2d(&self) -> Pos2D<T> {
        self.position.clone()
    }
}

impl<T: FixedPoint> GetPos2D for Remote<T> {
    type BaseType = T;
    fn get_pos_2d(&self) -> Pos2D<T> {
        Pos2D {
            x: self.position.x.clone(),
            y: self.position.y.clone(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_pos2d() {
        let pos = Pos2D::<Decimal>::new(
            Decimal::from_str("1.0").unwrap(),
            Decimal::from_str("2.0").unwrap(),
        );
        assert_eq!(pos.x, Decimal::from_str("1.0").unwrap());
        assert_eq!(pos.y, Decimal::from_str("2.0").unwrap());
    }
    #[test]
    fn test_pos3d() {
        let pos = Pos3D::<Decimal>::new(
            Decimal::from_str("1.0").unwrap(),
            Decimal::from_str("2.0").unwrap(),
            Decimal::from_str("3.0").unwrap(),
        );
        assert_eq!(pos.x, Decimal::from_str("1.0").unwrap());
        assert_eq!(pos.y, Decimal::from_str("2.0").unwrap());
        assert_eq!(pos.height, Decimal::from_str("3.0").unwrap());
        //test for bigint
        let pos = Pos3D::<BigInt>::new(
            BigInt::from(10000),
            BigInt::from(20000),
            BigInt::from(30000),
        );
        assert_eq!(pos.x, BigInt::from(10000));
        assert_eq!(pos.y, BigInt::from(20000));
        assert_eq!(pos.height, BigInt::from(30000));
    }
    #[test]
    fn test_pos2d_from_f64() {
        let pos = Pos2D::<Decimal>::new_from_f64(1.0, 2.0).unwrap();
        assert_eq!(pos.x, Decimal::from_str("1.0").unwrap());
        assert_eq!(pos.y, Decimal::from_str("2.0").unwrap());
        //test for bigint
        let pos = Pos2D::<BigInt>::new_from_f64(1.0, 2.0, &BigInt::from(10000)).unwrap();
        assert_eq!(pos.x, BigInt::from(10000));
        assert_eq!(pos.y, BigInt::from(20000));
    }
    #[test]
    fn test_pos3d_from_f64() {
        let pos = Pos3D::<Decimal>::new_from_f64(0.1, 0.2, 0.25).unwrap();
        assert_eq!(pos.x, Decimal::from_str("0.1").unwrap());
        assert_eq!(pos.y, Decimal::from_str("0.2").unwrap());
        assert_eq!(pos.height, Decimal::from_str("0.25").unwrap());
        //test for bigint
        let pos = Pos3D::<BigInt>::new_from_f64(0.1, 0.2, 0.25, &BigInt::from(10000)).unwrap();
        assert_eq!(pos.x, BigInt::from(1000));
        assert_eq!(pos.y, BigInt::from(2000));
        assert_eq!(pos.height, BigInt::from(2500));
    }
    #[test]
    fn test_pos2d_to_decimal() {
        let pos = Pos2D::<BigInt>::new_from_f64(0.0001, 0.0002, &BigInt::from(10000)).unwrap();
        let pos = pos.to_decimal(4).unwrap();
        assert_eq!(pos.x, Decimal::from_str("0.0001").unwrap());
        assert_eq!(pos.y, Decimal::from_str("0.0002").unwrap());
    }
    #[test]
    fn test_pos3d_to_decimal() {
        let pos =
            Pos3D::<BigInt>::new_from_f64(0.0001, 0.0002, 0.00025, &BigInt::from(100000)).unwrap();
        let pos = pos.to_decimal(5).unwrap();
        assert_eq!(pos.x, Decimal::from_str("0.0001").unwrap());
        assert_eq!(pos.y, Decimal::from_str("0.0002").unwrap());
        assert_eq!(pos.height, Decimal::from_str("0.00025").unwrap());
    }
}
