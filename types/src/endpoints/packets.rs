use std::str::FromStr;

use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};

use crate::{Error, MerkleAble};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub data: Vec<u8>,
}
impl FromStr for Packet {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            data: s.as_bytes().to_vec(),
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packets {
    // data must be sorted by seq and must be continuous
    pub data: Vec<Option<Packet>>,
}
#[derive(Debug, Clone)]
pub struct CompletePackets {
    pub data: Vec<Packet>,
}
impl MerkleAble for CompletePackets {
    fn merkle_tree(&self) -> Result<MerkleTree<Sha256>, Error> {
        if self.data.len() == 0 {
            return Err(Error::EmptyMerkleTreeErr);
        }
        let leaves = self
            .data
            .iter()
            .map(|x| Sha256::hash(x.data.as_slice()))
            .collect::<Vec<_>>();
        Ok(MerkleTree::<Sha256>::from_leaves(&leaves))
    }
}
impl MerkleAble for Packets {
    fn merkle_tree(&self) -> Result<MerkleTree<Sha256>, Error> {
        if self.data.len() == 0 {
            return Err(Error::EmptyMerkleTreeErr);
        }
        let leaves = self
            .data
            .iter()
            .map(|x| match x {
                Some(x) => Sha256::hash(x.data.as_slice()),
                None => Sha256::hash(&[]),
            })
            .collect::<Vec<_>>();
        Ok(MerkleTree::<Sha256>::from_leaves(&leaves))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_packet_from_str() {
        let packet = Packet::from_str("hello").unwrap();
        assert_eq!(packet.data, "hello".as_bytes());
    }
    #[test]
    fn test_packets_merkle_tree() {
        let packets = Packets {
            data: vec![
                Some(Packet {
                    data: "hello".as_bytes().to_vec(),
                }),
                Some(Packet {
                    data: "world".as_bytes().to_vec(),
                }),
            ],
        };
        let merkle_tree = packets.merkle_tree().unwrap();

        //7305db9b2abccd706c256db3d97e5ff48d677cfe4d3a5904afb7da0e3950e1e2
        assert_eq!(
            hex::encode(merkle_tree.root().unwrap()),
            "7305db9b2abccd706c256db3d97e5ff48d677cfe4d3a5904afb7da0e3950e1e2"
        );
    }
    #[test]
    fn test_packets_merkle_tree_empty() {
        let packets = Packets { data: vec![] };
        let merkle_tree = packets.merkle_tree();
        assert!(merkle_tree.is_err());
        let packets = Packets { data: vec![None] };
        let merkle_tree = packets.merkle_tree().unwrap();
        //e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            hex::encode(merkle_tree.root().unwrap()),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
    #[test]
    fn test_complete_packets_merkle_tree() {
        let complete_packets = CompletePackets {
            data: vec![
                Packet {
                    data: "hello".as_bytes().to_vec(),
                },
                Packet {
                    data: "world".as_bytes().to_vec(),
                },
            ],
        };
        let merkle_tree = complete_packets.merkle_tree().unwrap();

        //7305db9b2abccd706c256db3d97e5ff48d677cfe4d3a5904afb7da0e3950e1e2
        assert_eq!(
            hex::encode(merkle_tree.root().unwrap()),
            "7305db9b2abccd706c256db3d97e5ff48d677cfe4d3a5904afb7da0e3950e1e2"
        );
    }
}
