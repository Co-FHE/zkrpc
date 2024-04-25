use num_bigint::BigInt;
use types::Satellite;

pub struct PoX {
    satellites: Vec<Satellite<BigInt>>,
}
impl PoX {
    fn empty() -> Self {
        Self {
            satellites: Vec::new(),
        }
    }
    fn new(satellites: Vec<Satellite<BigInt>>) -> Self {
        Self { satellites }
    }
    fn add(&mut self, satellite: Satellite<BigInt>) {
        self.satellites.push(satellite)
    }
}
