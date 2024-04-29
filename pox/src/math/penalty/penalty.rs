use types::FixedPoint;

pub trait Penalty {
    type BaseType: FixedPoint;
    fn eval(&self, dist: Self::BaseType) -> Self::BaseType;
}
