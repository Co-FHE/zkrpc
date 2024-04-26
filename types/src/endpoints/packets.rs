#[derive(Debug, Clone)]
pub struct Packet {
    pub data: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct Packets {
    // data must be sorted by seq and must be continuous
    pub data: Vec<Option<Packet>>,
}
#[derive(Debug, Clone)]
pub struct CompletePackets {
    pub data: Vec<Packet>,
}
