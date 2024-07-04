use std::collections::HashMap;
use std::time::Instant;

use config::PoxConfig;

use halo2_proofs::pasta::Fp;
use hdrhistogram::{Histogram, SyncHistogram};
use num_bigint::{BigInt, ToBigInt};
use num_rational::Ratio;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleProof};
use rust_decimal::prelude::{FromPrimitive, Zero};
use rust_decimal::Decimal;
use tracing::{debug, warn};
use types::{
    Alpha, Error, FixedPoint, FixedPointInteger, GetPos2D, MerkleAble, MerkleComparison,
    MerkleProofStruct, Pos3D, Remote,
};
mod math;
use math::*;
mod tests;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use util::blockchain::address_brief;
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
    value_for_remote: T,
    proof: (Vec<u8>, Vec<u8>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoDRemoteResult<T: FixedPoint> {
    pub score: T,
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
pub struct PoFRemoteResult<T: FixedPoint> {
    pub value: T,
    pub terminal_results: Vec<PoFTerminalResult<T>>,
}
impl SerdeBinTrait for PoDRemoteResult<BigInt> {}
impl SerdeBinTrait for PoFRemoteResult<BigInt> {}
#[derive(Debug, Clone)]
pub struct PoX<P: Penalty<BaseType = BigInt>, ZK: zkt::ZkTraitHalo2<F = Fp>> {
    zk_prover: ZK,
    pub(crate) kernel: KernelKind<BigInt>,
    pub(crate) penalty: P,
    remote: Remote<BigInt>,
    pod_max_value: BigInt,
    cfg: PoxConfig,
}
use zkt::ZkTraitHalo2;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoFVerify {
    Success,
    Fail(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoDVerify {
    Success,
    Fail,
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
            value_for_remote: BigInt::zero(),
            proof: (vec![], vec![]),
        }
    }
}
impl PoDRemoteResult<BigInt> {
    pub fn new_from_results(
        results: Vec<PoDTerminalResult<BigInt>>,
        pod_max_value: BigInt,
    ) -> Self {
        let total_value = results
            .iter()
            .map(|r| r.value_for_remote.clone() * r.weight.clone())
            .sum::<BigInt>();
        let weight: BigInt = results.iter().map(|r| r.weight.clone()).sum();
        if weight.is_zero() {
            warn!("PoD: Total weight is zero, set value to zero");
            return PoDRemoteResult {
                score: BigInt::zero(),
                terminal_results: results,
            };
        }
        let value = Ratio::new(total_value.clone(), weight.clone()).to_integer();
        let score = (value.clone() - pod_max_value.clone()).max(BigInt::zero());
        debug!(message = "PoD Result", ?total_value, ?weight,remote_value=?value.clone(),remote_score=?score.clone());
        PoDRemoteResult {
            score,
            terminal_results: results,
        }
    }
    pub fn verify(&self) -> Vec<PoDVerify> {
        self.terminal_results
            .iter()
            .map(|r| {
                if ZKT::verify_proof(r.proof.0.clone(), r.proof.1.clone()) {
                    PoDVerify::Success
                } else {
                    PoDVerify::Fail
                }
            })
            .collect()
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
impl PoFRemoteResult<BigInt> {
    pub fn new_from_results(results: Vec<PoFTerminalResult<BigInt>>) -> Self {
        let total_value: BigInt = results.iter().map(|r| r.valid_packets_num.clone()).sum();
        debug!(message = "PoF Result", ?total_value);
        // if results.len() == 0 {
        //     warn!("PoF: No terminal results, set value to zero");
        //     return PoFRemoteResult {
        //         value: BigInt::zero(),
        //         terminal_results: results,
        //     };
        // }
        // let value = total_value / results.len();
        PoFRemoteResult {
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
impl<ZK> PoX<LinearPenalty<BigInt>, ZK>
where
    ZK: zkt::ZkTraitHalo2<F = Fp>,
{
    pub fn new(remote: Remote<BigInt>, zkp: ZK, cfg: &PoxConfig) -> Result<Self, Error> {
        let _span = tracing::debug_span!("PoX::new").entered();
        let mut terminals = remote.terminals.clone();

        let mut counts = HashMap::new();
        for t in &terminals {
            let count = counts.entry(t.address.clone()).or_insert(0);
            *count += 1;
        }
        terminals.retain(|t| counts[&t.address] == 1);
        debug!(
            message = format!(
                "remove duplicate terminals for Remote {}",
                address_brief(&remote.address)
            ),
            before = counts.len(),
            after = terminals.len()
        );
        // TODO: address may have lower case or upper case problem
        terminals.sort_by(|a, b| a.address.cmp(&b.address));
        let remote = Remote {
            address: remote.address.clone(),
            terminals,
            position: remote.position.clone(),
            remote_packets: remote.remote_packets.clone(),
            epoch: remote.epoch.clone(),
        };
        let pox = Self {
            kernel: match cfg.kernel.kernel_type {
                config::KernelTypeConfig::GaussianTaylor => KernelKind::GaussianTaylor(
                    Gaussian::<BigInt, GaussianTaylor>::from_pox_cfg(&cfg)?,
                ),
                config::KernelTypeConfig::Quadratic => {
                    KernelKind::Quadratic(Quadratic::<BigInt>::from_pox_cfg(&cfg)?)
                }
            },
            remote,
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
            cfg: cfg.clone(),
        };
        debug!(meesage="PoX",kernel=?pox.kernel,penalty=?pox.penalty,pod_max_value=?pox.pod_max_value);
        Ok(pox)
    }
    pub fn eval_pod(&self) -> PoDRemoteResult<BigInt> {
        let _span = tracing::debug_span!("eval_pod").entered();
        let mut coef_hist = SyncHistogram::<u64>::from(Histogram::new(3).unwrap());
        let mut rspr_hist = SyncHistogram::<u64>::from(Histogram::new(3).unwrap());
        let mut x_hist = SyncHistogram::<u64>::from(Histogram::new(3).unwrap());
        let mut y_hist = SyncHistogram::<u64>::from(Histogram::new(3).unwrap());
        let coor_to_u64 = |coor: &BigInt| -> u64 {
            let coor_offset: BigInt = FixedPointInteger::fixed_from_decimal(
                Decimal::from_u64(50000).unwrap(),
                self.cfg.coordinate_precision_bigint,
            )
            .map_or_else(
                |e| {
                    warn!("coor_to_u64 error: {}", e);
                    BigInt::zero()
                },
                |f| f,
            );
            let coor = coor + &coor_offset;
            let coor = coor.max(BigInt::zero());
            let coor = coor.min(coor_offset * 2);
            coor.fixed_magnitude_to_u64().map_or_else(
                |e| {
                    warn!("coor_to_u64 error: {}", e);
                    u64::max_value() / 4
                },
                |f| f,
            )
        };
        let u64_to_coor_f64 = |coor_u64: u64| -> f64 {
            coor_u64 as f64 / self.cfg.coordinate_precision_pow10() as f64 - 50000.0
        };
        let calc_coefx_start = Instant::now();
        let mut nearby_len_hist = Histogram::<u64>::new(3).unwrap();
        let coefx = self
            .remote
            .terminals
            .par_iter()
            .enumerate()
            .map(|(i, t1)| {
                let _ = rspr_hist
                    .recorder()
                    .record(t1.alpha.rspr.fixed_magnitude_to_u64().map_or_else(
                        |e| {
                            warn!("record rspr_hist error: {}", e);
                            u64::max_value() / 4
                        },
                        |f| f,
                    ))
                    .map_err(|e| {
                        warn!("record coef_hist error: {}", e);
                        e
                    });
                let mut coef_hist = coef_hist.recorder();
                let mut x_hist = x_hist.recorder();
                let mut y_hist = y_hist.recorder();
                (
                    self.remote
                        .terminals
                        .iter()
                        .filter_map(|t2| {
                            let coef = self.kernel.eval_numer(&t1.get_pos_2d(), &t2.get_pos_2d());
                            if coef.fixed_is_zero() {
                                None
                            } else {
                                let _ = coef_hist
                                    .record(coef.fixed_log_magnitude_to_u64().map_or_else(
                                        |e| {
                                            warn!("coef to_u64 error: {}", e);
                                            u64::max_value() / 4
                                        },
                                        |f| f,
                                    ))
                                    .map_err(|e| {
                                        warn!("record coef_hist error: {}", e);
                                        e
                                    });
                                let _ = x_hist.record(coor_to_u64(&t2.position.x)).map_err(|e| {
                                    warn!("record coef_hist error: {}", e);
                                    e
                                });
                                let _ = y_hist.record(coor_to_u64(&t2.position.y)).map_err(|e| {
                                    warn!("record coef_hist error: {}", e);
                                    e
                                });
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
            .collect::<Vec<(Vec<_>, Alpha<BigInt>, String)>>();
        debug!(calc_coefx_time = ?calc_coefx_start.elapsed());
        coef_hist.refresh();
        rspr_hist.refresh();
        x_hist.refresh();
        y_hist.refresh();
        // info! average & min & max & percentile
        coefx.iter().for_each(|(coefs, _, _)| {
            let _ = nearby_len_hist.record(coefs.len() as u64).map_err(|e| {
                warn!("record nearby_len_hist error: {}", e);
                e
            });
        });
        debug!(
            message = "nearby_len_hist",
            total = nearby_len_hist.len(),
            avg = nearby_len_hist.mean(),
            min = nearby_len_hist.min(),
            max = nearby_len_hist.max(),
            p25 = nearby_len_hist.value_at_quantile(0.25),
            p50 = nearby_len_hist.value_at_quantile(0.5),
            p75 = nearby_len_hist.value_at_quantile(0.75),
        );
        let pos = self
            .remote
            .position
            .to_decimal(self.cfg.coordinate_precision_bigint)
            .map_or_else(
                |e| {
                    warn!("fixed_to_u64 error: {}", e);
                    Pos3D::<Decimal> {
                        x: Decimal::zero(),
                        y: Decimal::zero(),
                        height: Decimal::zero(),
                    }
                },
                |f| f,
            );
        debug!(
            message = "remote position",
            x = pos.x.to_string(),
            y = pos.y.to_string(),
            z = pos.height.to_string(),
        );
        debug!(
            message = "x_hist",
            total = x_hist.len(),
            avg = u64_to_coor_f64(x_hist.mean() as u64),
            min = u64_to_coor_f64(x_hist.min()),
            max = u64_to_coor_f64(x_hist.max()),
            p25 = u64_to_coor_f64(x_hist.value_at_quantile(0.25)),
            p50 = u64_to_coor_f64(x_hist.value_at_quantile(0.5)),
            p75 = u64_to_coor_f64(x_hist.value_at_quantile(0.75)),
        );
        debug!(
            message = "y_hist",
            total = y_hist.len(),
            avg = u64_to_coor_f64(y_hist.mean() as u64),
            min = u64_to_coor_f64(y_hist.min()),
            max = u64_to_coor_f64(y_hist.max()),
            p25 = u64_to_coor_f64(y_hist.value_at_quantile(0.25)),
            p50 = u64_to_coor_f64(y_hist.value_at_quantile(0.5)),
            p75 = u64_to_coor_f64(y_hist.value_at_quantile(0.75)),
        );
        let multiplier_coor =
            self.cfg.coordinate_precision_pow10() as f64 * self.cfg.rspr_precision_pow10() as f64;
        let multiplier_rspr = self.cfg.rspr_precision_pow10() as f64;
        debug!(
            message = "log_coef_hist",
            total = coef_hist.len(),
            avg = coef_hist.mean() / multiplier_coor,
            min = coef_hist.min() as f64 / multiplier_coor,
            max = coef_hist.max() as f64 / multiplier_coor,
            p25 = coef_hist.value_at_quantile(0.25) as f64 / multiplier_coor,
            p50 = coef_hist.value_at_quantile(0.5) as f64 / multiplier_coor,
            p75 = coef_hist.value_at_quantile(0.75) as f64 / multiplier_coor,
        );
        debug!(
            message = "rspr_hist",
            total = rspr_hist.len(),
            avg = -rspr_hist.mean() / multiplier_rspr as f64,
            min = -1_f64 * rspr_hist.min() as f64 / multiplier_rspr,
            max = -1_f64 * rspr_hist.max() as f64 / multiplier_rspr,
            p25 = -1_f64 * rspr_hist.value_at_quantile(0.25) as f64 / multiplier_rspr,
            p50 = -1_f64 * rspr_hist.value_at_quantile(0.5) as f64 / multiplier_rspr,
            p75 = -1_f64 * rspr_hist.value_at_quantile(0.75) as f64 / multiplier_rspr,
        );
        // debug!("coefx: {:#?}", coefx);
        let mut diff_mag_hist = SyncHistogram::<u64>::from(Histogram::new(3).unwrap());
        let mut rspr_eval_hist = SyncHistogram::<u64>::from(Histogram::new(3).unwrap());
        let mut weight_mag_hist = SyncHistogram::<u64>::from(Histogram::new(3).unwrap());
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
                    let diff = diff.to_bigint().ok_or(Error::BigIntConversionErr(
                        diff.to_string(),
                        "to_bigint".to_owned(),
                    ))?;
                    let diff = Ratio::new(diff, total_weight.clone());
                    let _ = diff_mag_hist
                        .recorder()
                        .record(diff.to_integer().fixed_magnitude_to_u64().map_or_else(
                            |e| {
                                warn!("fixed_to_u64 error: {}", e);
                                u64::max_value() / 4
                            },
                            |f| f,
                        ))
                        .map_err(|e| {
                            warn!("record diff_mag_hist error: {}", e);
                            e
                        });
                    let value = Ratio::new(total_value.clone(), total_weight.clone());
                    let _ = rspr_eval_hist
                        .recorder()
                        .record(
                            (total_value / total_weight)
                                .fixed_magnitude_to_u64()
                                .map_or_else(
                                    |e| {
                                        warn!("fixed_to_u64 error: {}", e);
                                        u64::max_value() / 4
                                    },
                                    |f| f,
                                ),
                        )
                        .map_err(|e| {
                            warn!("record rspr_eval_hist error: {}", e);
                            e
                        });

                    let weight = self.penalty.eval(diff.to_integer());
                    let _ = weight_mag_hist
                        .recorder()
                        .record(weight.fixed_magnitude_to_u64().map_or_else(
                            |e| {
                                warn!("fixed_to_u64 error: {}", e);
                                u64::max_value() / 4
                            },
                            |f| f,
                        ))
                        .map_err(|e| {
                            warn!("record weight_mag_hist error: {}", e);
                            e
                        });
                    // debug!(
                    //     "PoD: address: {}, weight: {}, value: {}, binding: {}, diff: {}",
                    //     address, weight, value, binding, diff
                    // );
                    // info!(coefs_len = coefs.len(), xs_len = xs.len(), "zk input len");
                    let zkr = self
                        .zk_prover
                        .gen_proof(coefs, xs)
                        .map_err(|e| Error::ZeroKnownledgeProofErr(e.to_string()))?;
                    Ok(PoDTerminalResult {
                        weight,
                        value_for_remote: value.to_integer(),
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

        assert!(pod_result.len() == self.remote.terminals.len());
        rspr_eval_hist.refresh();
        diff_mag_hist.refresh();
        weight_mag_hist.refresh();
        debug!(
            message = "rspr_eval_hist",
            total = rspr_eval_hist.len(),
            avg = -rspr_eval_hist.mean() / multiplier_rspr,
            min = -1_f64 * rspr_eval_hist.min() as f64 / multiplier_rspr,
            max = -1_f64 * rspr_eval_hist.max() as f64 / multiplier_rspr,
            p25 = -1_f64 * rspr_eval_hist.value_at_quantile(0.25) as f64 / multiplier_rspr,
            p50 = -1_f64 * rspr_eval_hist.value_at_quantile(0.5) as f64 / multiplier_rspr,
            p75 = -1_f64 * rspr_eval_hist.value_at_quantile(0.75) as f64 / multiplier_rspr,
        );
        debug!(
            message = "diff_mag_hist",
            total = diff_mag_hist.len(),
            avg = diff_mag_hist.mean() / multiplier_rspr,
            min = diff_mag_hist.min() as f64 / multiplier_rspr,
            max = diff_mag_hist.max() as f64 / multiplier_rspr,
            p25 = diff_mag_hist.value_at_quantile(0.25) as f64 / multiplier_rspr,
            p50 = diff_mag_hist.value_at_quantile(0.5) as f64 / multiplier_rspr,
            p75 = diff_mag_hist.value_at_quantile(0.75) as f64 / multiplier_rspr,
        );
        debug!(
            message = "weight_mag_hist",
            total = weight_mag_hist.len(),
            avg = weight_mag_hist.mean(),
            min = weight_mag_hist.min() as f64,
            max = weight_mag_hist.max() as f64,
            p25 = weight_mag_hist.value_at_quantile(0.25) as f64,
            p50 = weight_mag_hist.value_at_quantile(0.5) as f64,
            p75 = weight_mag_hist.value_at_quantile(0.75) as f64,
        );
        PoDRemoteResult::new_from_results(pod_result, self.pod_max_value.clone())
    }
    pub fn eval_pof(&self) -> PoFRemoteResult<BigInt> {
        let _span = tracing::debug_span!("eval_pof").entered();
        let result = if let Some(remote_packets) = self.remote.remote_packets.as_ref() {
            let ref_merkle = remote_packets.merkle_tree();
            let ref_merkle = match ref_merkle {
                Ok(m) => m,
                Err(e) => {
                    warn!("PoF: Reference Merkle tree error: {}", e.to_string());
                    return PoFRemoteResult {
                        value: BigInt::zero(),
                        terminal_results: Vec::new(),
                    };
                }
            };
            debug!(
                "PoF: Reference Merkle tree root: {}",
                match ref_merkle.root() {
                    Some(r) => hex::encode(r),
                    None => "None".to_string(),
                }
            );
            let packet_len_hist = SyncHistogram::from(Histogram::<u64>::new(3).unwrap());
            let dropped_packet_len_hist = SyncHistogram::from(Histogram::<u64>::new(3).unwrap());
            let dropped_rate_hist = SyncHistogram::from(Histogram::<u64>::new(3).unwrap());
            let result = self
                .remote
                .terminals
                .par_iter()
                .map(|t| {
                    if let Some(terminal_packets) = t.terminal_packets.as_ref() {
                        let dropped_merkle = terminal_packets.merkle_tree()?;
                        let proof = ref_merkle
                            .comparison_proof_with_dropping_difference(&dropped_merkle)?;
                        let _ = packet_len_hist
                            .recorder()
                            .record(terminal_packets.data.len() as u64)
                            .map_err(|e| {
                                warn!("record packet_len_hist error: {}", e);
                                e
                            });
                        let _ = dropped_packet_len_hist
                            .recorder()
                            .record(proof.indices_to_prove.len() as u64)
                            .map_err(|e| {
                                warn!("record dropped_packet_len_hist error: {}", e);
                                e
                            });
                        let dropped_rate: u64 = (proof.indices_to_prove.len() as f64
                            / remote_packets.data.len() as f64
                            * 10000_f64) as u64;
                        let _ = dropped_rate_hist
                            .recorder()
                            .record(dropped_rate)
                            .map_err(|e| {
                                warn!("record dropped_rate_hist error: {}", e);
                                e
                            });

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
            debug!(
                message = "packet_len_hist",
                total = packet_len_hist.len(),
                avg = packet_len_hist.mean(),
                min = packet_len_hist.min(),
                max = packet_len_hist.max(),
                p25 = packet_len_hist.value_at_quantile(0.25),
                p50 = packet_len_hist.value_at_quantile(0.5),
                p75 = packet_len_hist.value_at_quantile(0.75),
            );
            debug!(
                message = "dropped_packet_len_hist",
                total = dropped_packet_len_hist.len(),
                avg = dropped_packet_len_hist.mean(),
                min = dropped_packet_len_hist.min(),
                max = dropped_packet_len_hist.max(),
                p25 = dropped_packet_len_hist.value_at_quantile(0.25),
                p50 = dropped_packet_len_hist.value_at_quantile(0.5),
                p75 = dropped_packet_len_hist.value_at_quantile(0.75),
            );
            debug!(
                message = "dropped_rate_hist",
                total = dropped_rate_hist.len(),
                avg = (dropped_rate_hist.mean() / 100.0).to_string() + "%",
                min = (dropped_rate_hist.min() as f64 / 100.0).to_string() + "%",
                max = (dropped_rate_hist.max() as f64 / 100.0).to_string() + "%",
                p25 = (dropped_rate_hist.value_at_quantile(0.25) as f64 / 100.0).to_string() + "%",
                p50 = (dropped_rate_hist.value_at_quantile(0.5) as f64 / 100.0).to_string() + "%",
                p75 = (dropped_rate_hist.value_at_quantile(0.75) as f64 / 100.0).to_string() + "%",
            );
            assert!(result.len() == self.remote.terminals.len());
            result
        } else {
            Vec::new()
        };
        PoFRemoteResult::new_from_results(result)
    }
}
