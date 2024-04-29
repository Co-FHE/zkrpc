use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};

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
impl CompletePackets {
    pub fn merkle_tree(&self) -> MerkleTree<Sha256> {
        let leaves = self
            .data
            .iter()
            .map(|x| Sha256::hash(x.data.as_slice()))
            .collect::<Vec<_>>();
        MerkleTree::<Sha256>::from_leaves(&leaves)
    }
}
impl Packets {
    pub fn merkle_tree(&self) -> MerkleTree<Sha256> {
        let leaves = self
            .data
            .iter()
            .map(|x| match x {
                Some(x) => Sha256::hash(x.data.as_slice()),
                None => Sha256::hash(&[]),
            })
            .collect::<Vec<_>>();
        MerkleTree::<Sha256>::from_leaves(&leaves)
    }
}
