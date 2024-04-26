use num_bigint::BigInt;
use types::Satellite;
mod math;
use math::*;

pub struct PoX {
    satellites: Vec<Satellite<BigInt>>,
}
impl PoX {
    pub fn empty() -> Self {
        Self {
            satellites: Vec::new(),
        }
    }
    pub fn new(satellites: Vec<Satellite<BigInt>>) -> Self {
        Self { satellites }
    }
    pub fn push(&mut self, satellite: Satellite<BigInt>) {
        self.satellites.push(satellite)
    }
}
