use rs_merkle::{algorithms::Sha256, Hasher, MerkleProof, MerkleTree};
use tracing::error;

use crate::Error;

pub struct MerkleProofStruct {
    pub reference_merkle_tree_root: [u8; 32],
    pub dropped_merkle_tree_root: [u8; 32],
    pub proof: Vec<u8>,
    pub indices_to_prove: Vec<usize>,
    pub leaves_to_prove: Vec<[u8; 32]>,
}
pub trait MerkleComparison {
    // compare the merkle tree of self with the merkle tree of other
    // return the indexes of the different leaves
    fn compare(&self, other: &Self) -> Result<Vec<usize>, Error>;
    fn comparison_proof(&self, other: &Self) -> Result<MerkleProofStruct, Error>;
}
impl MerkleComparison for MerkleTree<Sha256> {
    fn compare(&self, other: &Self) -> Result<Vec<usize>, Error> {
        let mut diff = vec![];
        let self_leaves = self.leaves().ok_or({
            error!("Couldn't get the leaves of the merkle tree");
            Error::MerkleTreeErr(format!(
                "Couldn't get the leaves of the merkle tree {:?}",
                self.root()
            ))
        })?;
        let other_leaves = other.leaves().ok_or({
            error!("Couldn't get the leaves of the merkle tree");
            Error::MerkleTreeErr(format!(
                "Couldn't get the leaves of the merkle tree {:?}",
                other.root()
            ))
        })?;
        for i in 0..self_leaves.len() {
            if self_leaves[i] != other_leaves[i] {
                diff.push(i);
            }
        }
        Ok(diff)
    }
    fn comparison_proof(&self, dropped_merkle_tree: &Self) -> Result<MerkleProofStruct, Error> {
        let diff = self.compare(dropped_merkle_tree)?;
        let binding = diff
            .clone()
            .into_iter()
            .map(|i| self.leaves().unwrap()[i])
            .collect::<Vec<_>>();
        let merkle_proof = self.proof(&diff);
        let merkle_root = self.root().ok_or({
            error!("Couldn't get the leaves of the merkle tree");
            Error::MerkleTreeErr(format!(
                "Couldn't get the leaves of the merkle tree {:?}",
                self.root()
            ))
        })?;
        let dropped_merkle_root = dropped_merkle_tree.root().ok_or({
            error!("Couldn't get the leaves of the merkle tree");
            Error::MerkleTreeErr(format!(
                "Couldn't get the leaves of the merkle tree {:?}",
                self.root()
            ))
        })?;
        Ok(MerkleProofStruct {
            reference_merkle_tree_root: merkle_root,
            dropped_merkle_tree_root: dropped_merkle_root,
            proof: merkle_proof.to_bytes(),
            indices_to_prove: diff,
            leaves_to_prove: binding,
        })
    }
}
impl MerkleProofStruct {
    pub fn empty() -> Self {
        MerkleProofStruct {
            reference_merkle_tree_root: [0; 32],
            dropped_merkle_tree_root: [0; 32],
            proof: vec![],
            indices_to_prove: vec![],
            leaves_to_prove: vec![],
        }
    }
    pub fn verify(&self) -> bool {
        let proof = MerkleProof::<Sha256>::try_from(self.proof.as_slice()).unwrap();
        if !proof.verify(
            self.reference_merkle_tree_root,
            &self.indices_to_prove,
            self.leaves_to_prove.as_slice(),
            self.indices_to_prove.len(),
        ) {
            return false;
        }
        let dropped_leaves = self
            .leaves_to_prove
            .iter()
            .map(|_| Sha256::hash(b""))
            .collect::<Vec<_>>();
        proof.verify(
            self.dropped_merkle_tree_root,
            &self.indices_to_prove,
            dropped_leaves.as_slice(),
            self.indices_to_prove.len(),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rs_merkle::algorithms::Sha256;
    use rs_merkle::{Hasher, MerkleProof, MerkleTree};

    #[test]
    fn test_compare_merkle_tree() {
        let leaves1 = vec![
            Sha256::hash(b"1"),
            Sha256::hash(b"2"),
            Sha256::hash(b"3"),
            Sha256::hash(b"4"),
            Sha256::hash(b"2"),
        ];
        let leaves2 = vec![
            Sha256::hash(b"1"),
            Sha256::hash(b"3"),
            Sha256::hash(b"5"),
            Sha256::hash(b"4"),
            Sha256::hash(b"2"),
        ];
        let merkle_tree1 = MerkleTree::<Sha256>::from_leaves(&leaves1);
        let merkle_tree2 = MerkleTree::<Sha256>::from_leaves(&leaves2);
        let diff = merkle_tree1.compare(&merkle_tree2).unwrap();
        assert_eq!(diff, vec![1, 2]);
    }
    #[test]
    fn test_proof() {
        let leaves = vec![
            Sha256::hash(b"0"),
            Sha256::hash(b"1"),
            Sha256::hash(b"2"),
            Sha256::hash(b"3"),
            Sha256::hash(b"4"),
            Sha256::hash(b"5"),
            Sha256::hash(b"6"),
            Sha256::hash(b"7"),
            Sha256::hash(b"8"),
            Sha256::hash(b"9"),
        ];
        let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
        let indices_to_prove = vec![3, 4, 7];
        let binding = indices_to_prove
            .clone()
            .into_iter()
            .map(|i| leaves[i])
            .collect::<Vec<_>>();
        let leaves_to_prove = binding.as_slice();

        let merkle_proof = merkle_tree.proof(&indices_to_prove);
        let merkle_root = merkle_tree
            .root()
            .ok_or("couldn't get the merkle root")
            .unwrap();
        // Serialize proof to pass it to the client
        let proof_bytes = merkle_proof.to_bytes();

        // Parse proof back on the client
        let proof: MerkleProof<Sha256> = MerkleProof::try_from(proof_bytes.as_slice()).unwrap();
        assert!(proof.verify(
            merkle_root,
            &indices_to_prove,
            leaves_to_prove,
            leaves.len()
        ));
        let dropped_leaves = vec![
            Sha256::hash(b"0"),
            Sha256::hash(b"1"),
            Sha256::hash(b"2"),
            Sha256::hash(b""),
            Sha256::hash(b""),
            Sha256::hash(b"5"),
            Sha256::hash(b"6"),
            Sha256::hash(b""),
            Sha256::hash(b"8"),
            Sha256::hash(b"9"),
        ];
        let dropped_merkle_tree = MerkleTree::<Sha256>::from_leaves(&dropped_leaves);
        let dropped_merkle_root = dropped_merkle_tree
            .root()
            .ok_or("couldn't get the merkle root")
            .unwrap();
        // Serialize proof to pass it to the client
        assert!(proof.verify(
            dropped_merkle_root,
            &indices_to_prove,
            leaves_to_prove
                .iter()
                .map(|_| Sha256::hash(b""))
                .collect::<Vec<_>>()
                .as_slice(),
            leaves.len(),
        ));
    }
}
