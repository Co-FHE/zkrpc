use types::FixedPoint;

pub trait Penalty<T: FixedPoint> {
    fn eval(&self, dist: T) -> T;
}
