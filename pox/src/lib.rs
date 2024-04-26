use std::collections::HashMap;

use halo2_proofs::{pasta::Fp, poly};
use num_bigint::{BigInt, ToBigInt};
use num_rational::Ratio;
use types::{Alpha, Error, FixedPoint, FixedPointInteger, Satellite, Terminal};
mod math;
use math::*;
use rayon::prelude::*;

struct PoDCoef<T: FixedPoint> {
    index: usize,
    coef: T,
    x: T,
}
pub struct PoDResult {
    diff: Ratio<BigInt>,
    proof: Vec<u8>,
}
pub struct PoX<K: Kernel<Pos2D<BigInt>, BigInt>, ZK: zkt::ZkTraitHalo2<Fp>> {
    zk_prover: ZK,
    kernel: K,
    satellite: Satellite<BigInt>,
}
impl<ZK> PoX<Quadratic<BigInt>, ZK>
where
    ZK: zkt::ZkTraitHalo2<Fp>,
{
    pub fn new(satellite: Satellite<BigInt>, kernel: Quadratic<BigInt>, zkp: ZK) -> Self {
        let mut terminals = satellite.terminals.clone();

        let mut counts = HashMap::new();
        for t in &terminals {
            let count = counts.entry(t.address.clone()).or_insert(0);
            *count += 1;
        }
        terminals.retain(|t| counts[&t.address] == 1);
        // TODO: address may have lower case or upper case problem
        terminals.sort_by(|a, b| a.address.cmp(&b.address));
        Self {
            kernel,
            satellite,
            zk_prover: zkp,
        }
    }
    pub fn eval(&self) -> Vec<Result<PoDResult, Error>> {
        let coefx: Vec<(Vec<_>, Alpha<BigInt>)> = self
            .satellite
            .terminals
            .par_iter()
            .enumerate()
            .map(|(i, t1)| {
                (
                    self.satellite
                        .terminals
                        .iter()
                        .filter_map(|t2| {
                            let coef = self.kernel.eval(&t1.get_pos(), &t2.get_pos());
                            if coef.fixed_is_zero() {
                                None
                            } else {
                                Some(PoDCoef {
                                    index: i,
                                    coef,
                                    x: t2.alpha.rspr.clone(),
                                })
                            }
                        })
                        .collect(),
                    t1.alpha.clone(),
                )
            })
            .collect();
        let pod_result: Vec<_> = coefx
            .par_iter()
            .map(|v| -> Result<PoDResult, Error> {
                let coefs: Result<Vec<_>, Error> = v.0.iter().map(|e| e.coef.to_fp()).collect();
                let xs: Result<Vec<_>, Error> = v.0.iter().map(|e| e.x.to_fp()).collect();
                let coefs = coefs?;
                let xs = xs?;
                let zkr = self
                    .zk_prover
                    .gen_proof(coefs, xs)
                    .map_err(|e| Error::ZeroKnownledgeProofErr(e.to_string()))?;
                let total_value: BigInt = v.0.iter().map(|cx| cx.coef.clone() * cx.x.clone()).sum();
                let total_weight: BigInt = v.0.iter().map(|cx| cx.coef.clone()).sum();
                let binding = total_weight.clone() * v.1.rspr.clone() - total_value;
                let diff = binding.magnitude();
                let diff = diff
                    .to_bigint()
                    .ok_or(Error::BigIntConversionErr(diff.to_string()))?;
                let diff = Ratio::new(diff, total_weight);
                Ok(PoDResult { diff, proof: zkr.1 })
            })
            .collect();
        pod_result
    }
}
