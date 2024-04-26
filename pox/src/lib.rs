use num_bigint::BigInt;
use types::{FixedPoint, Satellite, Terminal};
mod math;
use math::*;
use rayon::prelude::*;

struct PoDCoef<T: FixedPoint> {
    index: usize,
    coef: T,
    x: T,
}

pub struct PoX<K: Kernel<Pos2D<BigInt>, BigInt>> {
    kernel: K,
    satellite: Satellite<BigInt>,
}
impl PoX<Quadratic<BigInt>> {
    pub fn new(satellite: Satellite<BigInt>, kernel: Quadratic<BigInt>) -> Self {
        Self { kernel, satellite }
    }
    pub fn calc_coef_x(&self) -> Vec<Vec<PoDCoef<BigInt>>> {
        self.satellite
            .terminals
            .par_iter()
            .enumerate()
            .map(|(i, t1)| {
                self.satellite
                    .terminals
                    .iter()
                    .filter_map(|t2| {
                        let coef = self.kernel.eval(&t1.get_pos(), &t2.get_pos());
                        if coef.fixed_is_zero() {
                            None
                        } else {
                            Some(PoDCoef {
                                index: i,
                                coef,
                                x: t2.alpha.rspr.clone(),
                            })
                        }
                    })
                    .collect()
            })
            .collect()
    }
}
