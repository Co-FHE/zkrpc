use tracing::warn;
use types::FixedPoint;

use super::penalty;

// y = max - x
#[derive(Clone, Debug)]
pub struct LinearPenalty<T: FixedPoint> {
    pub max_diff: T,
}
impl<T: FixedPoint> penalty::Penalty for LinearPenalty<T> {
    type BaseType = T;
    fn eval(&self, diff: T) -> T {
        if diff > self.max_diff || diff < T::fixed_zero() {
            warn!(message = "invalid diff:", ?diff, ?self.max_diff);
            T::fixed_zero()
        } else {
            self.max_diff.clone() - diff
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::Penalty;

    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_linear_penalty_eval() {
        let penalty = LinearPenalty {
            max_diff: Decimal::from_str("0.1").unwrap(),
        };
        assert_eq!(
            penalty.eval(Decimal::from_str("0.05").unwrap()),
            Decimal::from_str("0.05").unwrap()
        );
        assert_eq!(
            penalty.eval(Decimal::from_str("0.15").unwrap()),
            Decimal::from_str("0.0").unwrap()
        );
        assert_eq!(
            penalty.eval(Decimal::from_str("-0.05").unwrap()),
            Decimal::from_str("0.0").unwrap()
        );
    }
}
