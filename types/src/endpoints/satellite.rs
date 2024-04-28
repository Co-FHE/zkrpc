use crate::{endpoints::terminal::Terminal, CompletePackets, FixedPoint, Packets, Pos2D, Pos3D};
use std::{collections::HashMap, fmt::format};
#[derive(Debug)]
pub struct Satellite<T: FixedPoint> {
    pub epoch: usize,
    pub address: String,
    pub position: Pos3D<T>,
    pub terminals: Vec<Terminal<T>>,
    // if option == None, it means the satellite has not sent packets
    pub satellite_packets: Option<CompletePackets>,
}
impl<T: FixedPoint> Satellite<T> {
    pub fn info(&self) -> String {
        format!(
            "{} {} {} {}",
            format!("Satellite {}", self.address),
            format!("Terminals: {}", self.terminals.len()),
            format!(
                "Satellite Packets: {}",
                match &self.satellite_packets {
                    Some(p) => p.data.len(),
                    None => 0,
                }
            ),
            format!(
                " Valid Terminal Packets: {}",
                self.terminals
                    .iter()
                    .map(|t| {
                        match &t.terminal_packets {
                            Some(p) => p.data.iter().filter(|p| p.is_some()).count().to_string(),
                            None => "0".to_owned(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        )
    }
}
