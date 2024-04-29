use std::str::FromStr;

use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};

use crate::{Error, MerkleAble};

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
        print!("{:?}", leaves);
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
