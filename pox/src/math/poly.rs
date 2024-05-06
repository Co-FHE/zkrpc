use types::FixedPoint;
#[allow(dead_code)]

pub struct LinearPolynomial<T: FixedPoint> {
    pub coefs: Vec<T>,
    pub xs: Vec<T>,
}
impl<T: FixedPoint> LinearPolynomial<T> {
    #[allow(dead_code)]

    pub fn prod(&self) -> T {
        let mut sum = T::fixed_zero();
        for i in 0..self.coefs.len() {
            sum = sum + self.coefs[i].clone() * self.xs[i].clone();
        }
        sum
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_linear_polynomial_prod() {
        let poly = LinearPolynomial {
            coefs: vec![
                Decimal::from_str("0.1").unwrap(),
                Decimal::from_str("0.2").unwrap(),
            ],
            xs: vec![
                Decimal::from_str("0.3").unwrap(),
                Decimal::from_str("0.4").unwrap(),
            ],
        };
        assert_eq!(poly.prod(), Decimal::from_str("0.11").unwrap());
    }
}
