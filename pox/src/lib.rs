use std::collections::HashMap;

use config::config::PoxConfig;
use halo2_proofs::{pasta::Fp, poly};
use num_bigint::{BigInt, ToBigInt};
use num_rational::Ratio;
use rs_merkle::{algorithms::Sha256, MerkleProof};
use rust_decimal::prelude::Zero;
use tracing::warn;
use types::{
    Alpha, Error, FixedPoint, FixedPointInteger, GetPos2D, MerkleComparison, MerkleProofStruct,
    Pos2D, Satellite,
};
mod math;
use math::*;
use rayon::prelude::*;

struct PoDCoef<T: FixedPoint> {
    index: usize,
    coef: T,
    x: T,
}
pub struct PoDTerminalResult<T: FixedPoint> {
    terminal_address: String,
    weight: T,
    value_for_satellite: T,
    proof: Vec<u8>,
}
pub struct PoDSatelliteResult<T: FixedPoint> {
    value: T,
    terminal_results: Vec<PoDTerminalResult<T>>,
}
pub struct PoFTerminalResult<T: FixedPoint> {
    terminal_address: String,
    valid_packets_num: T,
    proof: MerkleProofStruct,
    invalid_packets_num: T,
}
pub struct PoFSatelliteResult<T: FixedPoint> {
    value: T,
    terminal_results: Vec<PoFTerminalResult<T>>,
}
pub struct PoX<K: Kernel<Pos2D<BigInt>, BigInt>, P: Penalty<BigInt>, ZK: zkt::ZkTraitHalo2<Fp>> {
    zk_prover: ZK,
    kernel: K,
    penalty: P,
    satellite: Satellite<BigInt>,
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
            proof: vec![],
        }
    }
}
impl PoDSatelliteResult<BigInt> {
    pub fn new_from_results(results: Vec<PoDTerminalResult<BigInt>>) -> PoDSatelliteResult<BigInt> {
        let total_value = results.iter().map(|r| r.value_for_satellite.clone()).sum();
        let weight: BigInt = results.iter().map(|r| r.weight.clone()).sum();
        if weight.is_zero() {
            warn!("PoD: Total weight is zero, set value to zero");
            return PoDSatelliteResult {
                value: BigInt::zero(),
                terminal_results: results,
            };
        }
        PoDSatelliteResult {
            value: Ratio::new(total_value, weight).to_integer(),
            terminal_results: results,
        }
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
    pub fn new_from_results(results: Vec<PoFTerminalResult<BigInt>>) -> PoFSatelliteResult<BigInt> {
        let total_value: BigInt = results.iter().map(|r| r.valid_packets_num.clone()).sum();
        if results.len() == 0 {
            warn!("PoF: No terminal results, set value to zero");
            return PoFSatelliteResult {
                value: BigInt::zero(),
                terminal_results: results,
            };
        }
        let value = total_value / results.len();
        PoFSatelliteResult {
            value: value,
            terminal_results: results,
        }
    }
}
impl<ZK> PoX<Quadratic<BigInt>, LinearPenalty<BigInt>, ZK>
where
    ZK: zkt::ZkTraitHalo2<Fp>,
{
    pub fn new(satellite: Satellite<BigInt>, zkp: ZK, cfg: PoxConfig) -> Result<Self, Error> {
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
        let pod_result = coefx
            .par_iter()
            .map(
                |(coefs_x, alpha, address)| -> Result<PoDTerminalResult<BigInt>, Error> {
                    let coefs: Result<Vec<_>, Error> =
                        coefs_x.iter().map(|e| e.coef.to_fp()).collect();
                    let xs: Result<Vec<_>, Error> = coefs_x.iter().map(|e| e.x.to_fp()).collect();
                    let coefs = coefs?;
                    let xs = xs?;
                    let zkr = self
                        .zk_prover
                        .gen_proof(coefs, xs)
                        .map_err(|e| Error::ZeroKnownledgeProofErr(e.to_string()))?;
                    let total_value: BigInt = coefs_x
                        .iter()
                        .map(|cx| cx.coef.clone() * cx.x.clone())
                        .sum();
                    let total_weight: BigInt = coefs_x.iter().map(|cx| cx.coef.clone()).sum();
                    let binding = total_weight.clone() * alpha.rspr.clone() - total_value.clone();
                    let diff = binding.magnitude();
                    let diff = diff
                        .to_bigint()
                        .ok_or(Error::BigIntConversionErr(diff.to_string()))?;
                    let diff = Ratio::new(diff, total_weight.clone());
                    let value = Ratio::new(total_value, total_weight);

                    let weight = self.penalty.eval(diff.to_integer());
                    Ok(PoDTerminalResult {
                        weight,
                        value_for_satellite: value.to_integer(),
                        proof: zkr.1,
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
        PoDSatelliteResult::new_from_results(pod_result)
    }
    pub fn eval_pof(&self) -> PoFSatelliteResult<BigInt> {
        let result = if let Some(satellite_packets) = self.satellite.satellite_packets.as_ref() {
            let ref_merkle = satellite_packets.merkle_tree();
            let result = self
                .satellite
                .terminals
                .iter()
                .map(|t| {
                    if let Some(terminal_packets) = t.terminal_packets.as_ref() {
                        let dropped_merkle = terminal_packets.merkle_tree();
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
