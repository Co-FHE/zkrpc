use config::PoxConfig;
use num_bigint::BigInt;
use rust_decimal::Decimal;

use crate::{
    endpoints::terminal::Terminal, CompletePackets, EndPointFrom, Error, FixedPoint, Pos3D,
};

#[derive(Debug, Clone)]
pub struct Satellite<T: FixedPoint> {
    pub epoch: usize,
    pub address: String,
    pub position: Pos3D<T>,
    pub terminals: Vec<Terminal<T>>,
    // if option == None, it means the satellite has not sent packets
    pub satellite_packets: Option<CompletePackets>,
}

impl EndPointFrom<Satellite<Decimal>> for Satellite<BigInt> {
    fn from_with_config(value: Satellite<Decimal>, cfg: &PoxConfig) -> Result<Self, Error> {
        Ok(Self {
            epoch: value.epoch,
            address: value.address,
            position: Pos3D::<BigInt>::new_from_decimal(
                value.position.x,
                value.position.y,
                value.position.height,
                cfg.coordinate_precision_bigint,
            )?,
            terminals: value
                .terminals
                .iter()
                .map(|t| Terminal::<BigInt>::from_with_config(t.clone(), cfg))
                .collect::<Result<Vec<_>, _>>()?,
            satellite_packets: value.satellite_packets,
        })
    }
}
