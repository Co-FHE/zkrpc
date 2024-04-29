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
        ((self.x.clone() - target.x.clone()).fixed_sqr()
            + (self.y.clone() - target.y.clone()).fixed_sqr())
    }
}

pub trait Kernel {
    type BaseType: FixedPoint;
    type PosType: PosTrait<BaseType = Self::BaseType>;
    fn eval(&self, x1: &Self::PosType, x2: &Self::PosType) -> Self::BaseType;
}
