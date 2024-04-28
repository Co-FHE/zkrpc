use std::fmt;

use halo2_proofs::arithmetic::Field;
// To Mike:
// Note: traits.rs is provided as an example and feel free for editing.
// Need 3 functions: gen_proof, verify_proof, setup

// Target of zk is: without disclosing the parameters of the terminal : coordinates and alpha
// (not disclosing means not having to submit them to the chain)
// it is able to correctly calculate the weight of each terminal.

//TODO: define error here

#[derive(Debug)]
pub struct Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "zkproof error")
    }
}
impl std::error::Error for Error {}

pub trait ZkTraitHalo2<F: Field>: std::marker::Sync {
    // coef \dot x = a
    fn gen_proof(
        &self,
        coef: Vec<F>,
        x: Vec<F>,
        // TODO: add other parameters
        // e.g. setup parameters
    ) -> Result<(Vec<u8>, Vec<u8>), Error>;
    // TODO: add verify_proof function
    // TODO: add setup function
}
