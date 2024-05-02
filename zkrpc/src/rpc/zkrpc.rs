pub mod pb {
    tonic::include_proto!("grpc.zkrpc.service");
}
use config::{Config, DaLayerConfig, RpcConfig};
use da_layer::{satellite, DaLayerTrait, MockLocalDB};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use flate2::{read, read::ZlibDecoder, write};
use halo2_proofs::pasta::Fp;
use num_bigint::BigInt;
use pb::*;
use pox::{PoDSatelliteResult, PoFSatelliteResult};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Read;
use std::{collections::HashMap, io::Write};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{debug, debug_span, error, info, info_span, Instrument};
use types::{EndPointFrom, Satellite};
use util::blockchain::address_brief;
use util::compressor::BrotliCompressor;
use util::serde_bin::SerdeBinTrait;
use zkt::{ZkTraitHalo2, ZKT};
#[derive(Debug, Clone)]
pub struct ZkRpcServer {
    pub addr: String,
    pub db: MockLocalDB,
    pub cfg: Config,
}

#[tonic::async_trait]
impl pb::zk_service_server::ZkService for ZkRpcServer {
    async fn gen_proof(
        &self,
        request: Request<ZkGenProofRequest>,
    ) -> Result<Response<ZkGenProofResponse>, Status> {
        let ip = request
            .remote_addr()
            .map_or("unknow".to_string(), |addr| addr.ip().to_string());
        let zk_request = request.into_inner();
        let prover_address = zk_request.prover_address.clone();
        let satellite_address = zk_request.satellite_address.clone();
        let epoch_for_proof = zk_request.epoch_for_proof;
        let block_height_from_for_proof = zk_request.block_height_from_for_proof;
        let block_height_to_for_proof = zk_request.block_height_to_for_proof;

        async move {
            info!(message = "Received zk proof, fetching data");
            let satellite_address = zk_request.satellite_address;
            let mut satellite = self
                .db
                .fetch_satellite_with_terminals_block_from_to(
                    satellite_address.as_ref(),
                    block_height_from_for_proof as u64,
                    block_height_to_for_proof as u64,
                )
                .instrument(info_span!("fetch_satellite_with_terminals_block_from_to"))
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
            info!(block_found = satellite.len());
            let block_heights = satellite.iter().map(|(k, _)| k).collect::<Vec<_>>();
            debug!(block_heights = ?block_heights);
            satellite.sort_by(|a, b| a.0.cmp(&b.0));
            if satellite.is_empty() {
                return Err(Status::not_found("No satellite found"));
            }
            info!(
                message = "use the satellite with min height",
                height = satellite[0].0
            );
            let satellite = satellite[0].1.clone();
            let satellite = Satellite::from_with_config(satellite, &self.cfg.pox).map_err(|e| {
                Status::internal(format!("Error converting Satellite: {}", e.to_string()))
            })?;
            let zkp = ZKT {};

            let pox = pox::PoX::new(satellite, zkp, &self.cfg.pox)
                .map_err(|e| Status::internal(format!("Error creating PoX: {}", e.to_string())))?;
            info!("evaluating PoD");
            let pod = pox.eval_pod();
            info!("evaluating PoF");
            let pof = pox.eval_pof();
            info!("compressing PoD and PoF");
            let pod_s = pod
                .serialize_compress::<BrotliCompressor>(&self.cfg.compressor)
                .map_err(|e| {
                    Status::internal(format!("Error serializing PoD: {}", e.to_string()))
                })?;
            let pof_s = pof
                .serialize_compress::<BrotliCompressor>(&self.cfg.compressor)
                .map_err(|e| {
                    Status::internal(format!("Error serializing PoF: {}", e.to_string()))
                })?;
            info!("PoD and PoF compressed");
            let mut pof_hashmap = HashMap::new();
            pof.terminal_results.iter().for_each(|t| {
                pof_hashmap.insert(
                    t.terminal_address.clone(),
                    t.invalid_packets_num.clone() + t.valid_packets_num.clone(),
                );
            });
            let response = ZkGenProofResponse {
                alpha_proof_merkle_root: hex::encode(pod_s),
                beta_proof_merkle_root: hex::encode(pof_s),
                satellite_alpha_weight: pod.value.to_string().parse::<u64>().map_err(|e| {
                    Status::internal(format!(
                        "Error parsing satellite_alpha_weight: {}",
                        e.to_string()
                    ))
                })?,
                satellite_beta_weight: pof.value.to_string().parse::<u64>().map_err(|e| {
                    Status::internal(format!(
                        "Error parsing satellite_beta_weight: {}",
                        e.to_string()
                    ))
                })?,
                terminal_weights: pod
                    .terminal_results
                    .iter()
                    .map(|t| -> Result<ZkWeight, Status> {
                        Ok(ZkWeight {
                            address: t.terminal_address.clone(),
                            alpha_weight: t.weight.to_string().parse::<u64>().map_err(|e| {
                                Status::internal(format!(
                                    "Error parsing terminal alpha_weight: {}",
                                    e.to_string()
                                ))
                            })?,
                            beta_weight: {
                                let r = pof_hashmap.get(&t.terminal_address);
                                match r {
                                    Some(v) => v.to_string().parse::<u64>().map_err(|e| {
                                        Status::internal(format!(
                                            "Error parsing terminal beta_weight: {}",
                                            e.to_string()
                                        ))
                                    })?,
                                    None => 0,
                                }
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, Status>>()?,
            };
            Ok(Response::new(response))
        }
        .instrument(info_span!(
            "gen_proof",
            ip,
            s_addr = address_brief(satellite_address),
            prover = address_brief(prover_address),
            epoch = epoch_for_proof,
            from = block_height_from_for_proof,
            to = block_height_to_for_proof
        ))
        .await
    }
    async fn verify_proof(
        &self,
        request: Request<ZkVerifyProofRequest>,
    ) -> Result<Response<ZkVerifyProofResponse>, Status> {
        let ip = request
            .remote_addr()
            .map_or("unknow".to_string(), |addr| addr.ip().to_string());
        let zk_request = request.into_inner();
        let prover_address = zk_request.prover_address.clone();
        let satellite_address = zk_request.satellite_address.clone();
        let epoch_for_proof = zk_request.epoch_for_proof;
        let block_height_from_for_proof = zk_request.block_height_from_for_proof;
        let block_height_to_for_proof = zk_request.block_height_to_for_proof;
        let mut hasher = DefaultHasher::new();
        zk_request.alpha_proof_merkle_root.hash(&mut hasher);
        let alpha_root_hash = hasher.finish();
        let mut hasher = DefaultHasher::new();
        zk_request.beta_proof_merkle_root.hash(&mut hasher);
        let beta_root_hash = hasher.finish();
        async move {
            info!(message = "Received zk verification request");
            let pod_s = hex::decode(zk_request.alpha_proof_merkle_root).map_err(|e| {
                Status::internal(format!(
                    "Error decoding alpha_proof_merkle_root: {}",
                    e.to_string()
                ))
            })?;
            let pof_s = hex::decode(zk_request.beta_proof_merkle_root).map_err(|e| {
                Status::internal(format!(
                    "Error decoding beta_proof_merkle_root: {}",
                    e.to_string()
                ))
            })?;
            let _pod = PoDSatelliteResult::decompress_deserialize(&pod_s, &self.cfg.compressor)
                .map_err(|e| {
                    Status::internal(format!("Error deserializing PoD: {}", e.to_string()))
                })?;
            let pof = PoFSatelliteResult::decompress_deserialize(&pof_s, &self.cfg.compressor)
                .map_err(|e| {
                    Status::internal(format!("Error deserializing PoF: {}", e.to_string()))
                })?;

            let verf = pof.verify();
            let pof_verf = verf.iter().all(|x| *x == pox::PoFVerify::Success);
            let success = verf
                .iter()
                .enumerate()
                .filter_map(|(i, r)| match r {
                    pox::PoFVerify::Success => Some(()),
                    pox::PoFVerify::Fail(f) => {
                        error!(message = format!("Verification {} failed", i), reason = f);
                        None
                    }
                })
                .count();
            info!("Verification result {}/{}", success, verf.len());
            let response = ZkVerifyProofResponse { is_valid: pof_verf };
            Ok(Response::new(response))
        }
        .instrument(info_span!(
            "verify_proof",
            ip,
            s_addr = %address_brief(satellite_address),
            prover = %address_brief(prover_address),
            epoch = epoch_for_proof,
            from = block_height_from_for_proof,
            to = block_height_to_for_proof,
            ahash = %format!("{:x}", alpha_root_hash),
            bhash = %format!("{:x}", beta_root_hash)
        ))
        .await
    }
}
impl ZkRpcServer {
    pub async fn new(cfg: &Config) -> color_eyre::Result<Self> {
        Ok(Self {
            addr: format!("{}:{}", cfg.rpc.rpc_host, cfg.rpc.rpc_port),
            db: MockLocalDB::new(&cfg.da_layer)
                .instrument(debug_span!("init_db"))
                .await?,
            cfg: cfg.clone(),
        })
    }
    pub async fn start(&self) -> color_eyre::Result<()> {
        let t: ZkRpcServer = self.clone();
        Server::builder()
            .add_service(pb::zk_service_server::ZkServiceServer::new(t))
            .serve(self.addr.parse()?)
            .await?;
        Ok(())
    }
}
