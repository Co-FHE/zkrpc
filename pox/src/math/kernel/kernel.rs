use types::{Error, FixedPoint, Pos2D};

pub trait PosTrait<T: FixedPoint> {
    fn dist(&self, target: &Self) -> Result<T, Error>;
    fn dist_sqr(&self, target: &Self) -> T;
}
impl<T> PosTrait<T> for Pos2D<T>
where
    T: FixedPoint,
{
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

pub trait Kernel<P: PosTrait<T>, T: FixedPoint> {
    fn eval(&self, x1: &P, x2: &P) -> T;
}
