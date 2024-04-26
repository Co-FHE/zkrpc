use types::{FixedPoint, FixedPointDecimal, FixedPointInteger};

pub struct LinearPolynomial<T: FixedPoint> {
    pub coefs: Vec<T>,
    pub xs: Vec<T>,
}
impl<T: FixedPoint> LinearPolynomial<T> {
    pub fn prod(&self) -> T {
        let mut sum = T::zero();
        for i in 0..self.coefs.len() {
            sum = sum + self.coefs[i].clone() * self.xs[i].clone();
        }
        sum
    }
}
