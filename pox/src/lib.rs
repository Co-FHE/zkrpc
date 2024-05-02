use std::collections::HashMap;

use config::PoxConfig;
use halo2_proofs::{pasta::Fp, poly};
use num_bigint::{BigInt, ToBigInt};
use num_rational::Ratio;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleProof};
use rust_decimal::prelude::Zero;
use tracing::{debug, warn};
use tracing::{error, info};
use types::{
    Alpha, Error, FixedPoint, FixedPointInteger, GetPos2D, MerkleAble, MerkleComparison,
    MerkleProofStruct, Pos2D, Satellite,
};
mod math;
use math::*;
mod tests;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use util::serde_bin::SerdeBinTrait;
use zkt::ZKT;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PoDCoef<T: FixedPoint> {
    index: usize,
    coef: T,
    x: T,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoDTerminalResult<T: FixedPoint> {
    pub terminal_address: String,
    pub weight: T,
    value_for_satellite: T,
    proof: (Vec<u8>, Vec<u8>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoDSatelliteResult<T: FixedPoint> {
    pub value: T,
    pub terminal_results: Vec<PoDTerminalResult<T>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoFTerminalResult<T: FixedPoint> {
    pub terminal_address: String,
    pub valid_packets_num: T,
    proof: MerkleProofStruct,
    pub invalid_packets_num: T,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoFSatelliteResult<T: FixedPoint> {
    pub value: T,
    pub terminal_results: Vec<PoFTerminalResult<T>>,
}
use util::compressor::CompressorTrait;
impl SerdeBinTrait for PoDSatelliteResult<BigInt> {}
impl SerdeBinTrait for PoFSatelliteResult<BigInt> {}
#[derive(Debug, Clone)]
pub struct PoX<
    K: Kernel<PosType = Pos2D<BigInt>, BaseType = BigInt>,
    P: Penalty<BaseType = BigInt>,
    ZK: zkt::ZkTraitHalo2<F = Fp>,
> {
    zk_prover: ZK,
    pub(crate) kernel: K,
    pub(crate) penalty: P,
    satellite: Satellite<BigInt>,
    pod_max_value: BigInt,
}
use zkt::ZkTraitHalo2;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoFVerify {
    Success,
    Fail(String),
}
impl PoDTerminalResult<BigInt> {
    pub fn new_empty_for_err(address: String, err: Error) -> PoDTerminalResult<BigInt> {
        warn!(
            "Terminal PoD {} error: {}, gernerate empty result",
            address,
            err.to_string()
        );
        PoDTerminalResult {
            terminal_address: address,
            weight: BigInt::zero(),
            value_for_satellite: BigInt::zero(),
            proof: (vec![], vec![]),
        }
    }
}
impl PoDSatelliteResult<BigInt> {
    pub fn new_from_results(
        results: Vec<PoDTerminalResult<BigInt>>,
        pod_max_value: BigInt,
    ) -> Self {
        let total_value = results
            .iter()
            .map(|r| r.value_for_satellite.clone() * r.weight.clone())
            .sum();
        let weight: BigInt = results.iter().map(|r| r.weight.clone()).sum();
        if weight.is_zero() {
            warn!("PoD: Total weight is zero, set value to zero");
            return PoDSatelliteResult {
                value: BigInt::zero(),
                terminal_results: results,
            };
        }
        let mut value = Ratio::new(total_value, weight).to_integer() - pod_max_value;
        if value.fixed_is_negative() {
            value = BigInt::zero();
        }
        PoDSatelliteResult {
            value,
            terminal_results: results,
        }
    }
    pub fn verify(&self) -> bool {
        self.terminal_results
            .iter()
            .all(|r| ZKT::verify_proof(r.proof.0.clone(), r.proof.1.clone()))
    }
}
impl PoFTerminalResult<BigInt> {
    pub fn new_empty_for_err(address: String, err: Error) -> PoFTerminalResult<BigInt> {
        warn!(
            "Terminal PoF {} error: {}, gernerate empty result",
            address,
            err.to_string()
        );
        PoFTerminalResult {
            terminal_address: address,
            valid_packets_num: BigInt::zero(),
            invalid_packets_num: BigInt::zero(),

            proof: MerkleProofStruct::empty(),
        }
    }
}
impl PoFSatelliteResult<BigInt> {
    pub fn new_from_results(results: Vec<PoFTerminalResult<BigInt>>) -> Self {
        let total_value: BigInt = results.iter().map(|r| r.valid_packets_num.clone()).sum();
        // if results.len() == 0 {
        //     warn!("PoF: No terminal results, set value to zero");
        //     return PoFSatelliteResult {
        //         value: BigInt::zero(),
        //         terminal_results: results,
        //     };
        // }
        // let value = total_value / results.len();
        PoFSatelliteResult {
            value: total_value,
            terminal_results: results,
        }
    }
    pub fn verify(&self) -> Vec<PoFVerify> {
        self.terminal_results
            .iter()
            .map(|r| {
                if r.proof.indices_to_prove.len()!=r.proof.leaves_to_prove.len(){
                    return PoFVerify::Fail(format!(
                        "PoF: Terminal {} proof verify failed: indices_to_prove.len()!=leaves_to_prove.len()",
                        r.terminal_address
                    ));
                }
                if r.proof.indices_to_prove.len() == 0 {
                    if r.valid_packets_num.is_zero() && r.invalid_packets_num.is_zero() {
                        return PoFVerify::Fail(format!(
                            "PoF: Terminal {} Empty proof",
                            r.terminal_address
                        ));
                    } else {
                        return PoFVerify::Success;
                    }
                }
                if BigInt::from(r.proof.indices_to_prove.len())!=r.invalid_packets_num{
                    return PoFVerify::Fail(format!(
                        "PoF: Terminal {} proof verify failed: indices_to_prove.len()!=invalid_packets_num",
                        r.terminal_address
                    ));
                }
                let proof: Result<MerkleProof<Sha256>, rs_merkle::Error> =
                    MerkleProof::try_from(r.proof.proof.as_slice());
                match proof {
                    Ok(proof) => {
                        if !proof.verify(
                            r.proof.reference_merkle_tree_root,
                            &r.proof.indices_to_prove,
                            r.proof.leaves_to_prove.as_slice(),
                            r.proof.total_leaves_count,
                        ) {
                            return PoFVerify::Fail(format!(
                                "Reference Merkle tree verify failed for terminal {}",
                                r.terminal_address
                            ));
                        }
                        if !proof.verify(
                            r.proof.dropped_merkle_tree_root,
                            &r.proof.indices_to_prove,
                            r.proof
                                .leaves_to_prove
                                .iter()
                                .map(|_| Sha256::hash(vec![].as_slice()))
                                .collect::<Vec<_>>()
                                .as_slice(),
                                r.proof.total_leaves_count,
                        ) {
                            return PoFVerify::Fail(format!(
                                "Dropped Merkle tree verify failed for terminal {}",
                                r.terminal_address
                            ));
                        }
                        PoFVerify::Success
                    }
                    Err(e) => PoFVerify::Fail(format!(
                        "PoF: Terminal {} proof verify failed: {}",
                        r.terminal_address,
                        e.to_string()
                    )),
                }
            })
            .collect()
    }
}
impl<ZK> PoX<Quadratic<BigInt>, LinearPenalty<BigInt>, ZK>
where
    ZK: zkt::ZkTraitHalo2<F = Fp>,
{
    pub fn new(satellite: Satellite<BigInt>, zkp: ZK, cfg: &PoxConfig) -> Result<Self, Error> {
        let mut terminals = satellite.terminals.clone();

        let mut counts = HashMap::new();
        for t in &terminals {
            let count = counts.entry(t.address.clone()).or_insert(0);
            *count += 1;
        }
        terminals.retain(|t| counts[&t.address] == 1);
        // TODO: address may have lower case or upper case problem
        terminals.sort_by(|a, b| a.address.cmp(&b.address));
        Ok(Self {
            kernel: Quadratic {
                max_dis_sqr: BigInt::fixed_from_decimal(
                    cfg.kernel.quadratic.max_dis_sqr,
                    cfg.cooridnate_precision_bigint * 2,
                )?,
            },
            satellite,
            zk_prover: zkp,
            penalty: LinearPenalty {
                max_diff: BigInt::fixed_from_decimal(
                    cfg.penalty.max_diff,
                    cfg.rspr_precision_bigint,
                )?,
            },
            pod_max_value: BigInt::fixed_from_decimal(
                cfg.pod_max_value,
                cfg.rspr_precision_bigint,
            )?,
        })
    }
    pub fn eval_pod(&self) -> PoDSatelliteResult<BigInt> {
        let coefx: Vec<(Vec<_>, Alpha<BigInt>, String)> = self
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
                            let coef = self.kernel.eval(&t1.get_pos_2d(), &t2.get_pos_2d());
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
                    t1.address.clone(),
                )
            })
            .collect();
        // debug!("coefx: {:#?}", coefx);
        let pod_result = coefx
            .par_iter()
            .map(
                |(coefs_x, alpha, address)| -> Result<PoDTerminalResult<BigInt>, Error> {
                    let coefs: Result<Vec<_>, Error> =
                        coefs_x.iter().map(|e| e.coef.to_fp()).collect();
                    let xs: Result<Vec<_>, Error> = coefs_x.iter().map(|e| e.x.to_fp()).collect();
                    let coefs = coefs?;
                    let xs = xs?;
                    let total_value: BigInt = coefs_x
                        .iter()
                        .map(|cx| cx.coef.clone() * cx.x.clone())
                        .sum();
                    let total_weight: BigInt = coefs_x.iter().map(|cx| cx.coef.clone()).sum();
                    // debug!(
                    //     "address: {}, total_value: {}, total_weight: {}",
                    //     address, total_value, total_weight
                    // );
                    let binding = total_weight.clone() * alpha.rspr.clone() - total_value.clone();
                    let diff = binding.magnitude();
                    let diff = diff
                        .to_bigint()
                        .ok_or(Error::BigIntConversionErr(diff.to_string()))?;
                    let diff = Ratio::new(diff, total_weight.clone());
                    let value = Ratio::new(total_value, total_weight);

                    let weight = self.penalty.eval(diff.to_integer());
                    // debug!(
                    //     "PoD: address: {}, weight: {}, value: {}, binding: {}, diff: {}",
                    //     address, weight, value, binding, diff
                    // );
                    info!(coefs_len = coefs.len(), xs_len = xs.len(), "zk input len");
                    let zkr = self
                        .zk_prover
                        .gen_proof(coefs, xs)
                        .map_err(|e| Error::ZeroKnownledgeProofErr(e.to_string()))?;
                    Ok(PoDTerminalResult {
                        weight,
                        value_for_satellite: value.to_integer(),
                        proof: zkr,
                        // proof: vec![],
                        terminal_address: address.clone(),
                    })
                },
            )
            .map(|r| match r {
                Ok(r) => r,
                Err(e) => PoDTerminalResult::new_empty_for_err("".to_string(), e),
            })
            .collect::<Vec<_>>();
        assert!(pod_result.len() == self.satellite.terminals.len());
        PoDSatelliteResult::new_from_results(pod_result, self.pod_max_value.clone())
    }
    pub fn eval_pof(&self) -> PoFSatelliteResult<BigInt> {
        let result = if let Some(satellite_packets) = self.satellite.satellite_packets.as_ref() {
            let ref_merkle = satellite_packets.merkle_tree();
            let ref_merkle = match ref_merkle {
                Ok(m) => m,
                Err(e) => {
                    return PoFSatelliteResult {
                        value: BigInt::zero(),
                        terminal_results: Vec::new(),
                    };
                }
            };
            let result = self
                .satellite
                .terminals
                .iter()
                .map(|t| {
                    if let Some(terminal_packets) = t.terminal_packets.as_ref() {
                        //TODO : if data is diff need remove packet manually
                        let dropped_merkle = terminal_packets.merkle_tree()?;
                        let proof = ref_merkle.comparison_proof(&dropped_merkle)?;
                        assert!(terminal_packets.data.len() >= proof.indices_to_prove.len());
                        Ok(PoFTerminalResult {
                            valid_packets_num: BigInt::from(
                                terminal_packets.data.len() - proof.indices_to_prove.len(),
                            ),
                            invalid_packets_num: BigInt::from(proof.indices_to_prove.len()),
                            proof: proof,
                            terminal_address: t.address.clone(),
                        })
                    } else {
                        Ok(PoFTerminalResult {
                            valid_packets_num: BigInt::zero(),
                            invalid_packets_num: BigInt::zero(),
                            proof: MerkleProofStruct::empty(),
                            terminal_address: t.address.clone(),
                        })
                    }
                })
                .map(|r| match r {
                    Ok(r) => r,
                    Err(e) => PoFTerminalResult::new_empty_for_err("".to_string(), e),
                })
                .collect::<Vec<_>>();
            assert!(result.len() == self.satellite.terminals.len());
            result
        } else {
            Vec::new()
        };
        PoFSatelliteResult::new_from_results(result)
    }
}
