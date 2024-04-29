use types::FixedPoint;

use super::penalty;

// y = max - x
pub struct LinearPenalty<T: FixedPoint> {
    pub max_diff: T,
}
impl<T: FixedPoint> penalty::Penalty<T> for LinearPenalty<T> {
    fn eval(&self, diff: T) -> T {
        if diff > self.max_diff || diff < T::fixed_zero() {
            T::fixed_zero()
        } else {
            self.max_diff.clone() - diff
        }
    }
}
