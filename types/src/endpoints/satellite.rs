use crate::{endpoints::terminal::Terminal, CompletePackets, FixedPoint, Packets};
use std::collections::HashMap;
pub struct Satellite<T: FixedPoint> {
    pub address: String,
    pub terminals: Vec<Terminal<T>>,
    pub terminal_packets: Vec<Option<Packets>>,
    pub satellite_packets: Vec<Option<CompletePackets>>,
}
