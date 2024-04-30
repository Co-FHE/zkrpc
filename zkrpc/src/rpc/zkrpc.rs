pub mod pb {
    tonic::include_proto!("grpc.zkrpc.service");
}
use std::{collections::HashMap, io::Write};

use config::config::{Config, DaLayerConfig, RpcConfig};
use da_layer::{DaLayerTrait, MockLocalDB};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use flate2::{read, read::ZlibDecoder, write};
use halo2_proofs::pasta::Fp;
use num_bigint::BigInt;
use pb::*;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{debug, error, info};
use types::{EndPointFrom, Satellite};
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
        let zk_request = request.into_inner();
        let prover_address = zk_request.prover_address.clone();
        let satellite_address = zk_request.satellite_address.clone();
        let epoch_for_proof = zk_request.epoch_for_proof;
        let block_height_from_for_proof = zk_request.block_height_from_for_proof;
        let block_height_to_for_proof = zk_request.block_height_to_for_proof;
        info!(
            "Received request: prover_address: {}, satellite_address: {}, epoch_for_proof: {}, block_height_from_for_proof: {}, block_height_to_for_proof: {}",
            prover_address, satellite_address, epoch_for_proof, block_height_from_for_proof, block_height_to_for_proof
        );
        debug!("Received request: {:?}", zk_request);
        let mut satellite = self
            .db
            .fetch_satellite_with_terminals_block_from_to(
                satellite_address.as_ref(),
                block_height_from_for_proof as u64,
                block_height_to_for_proof as u64,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        satellite.sort_by(|a, b| a.0.cmp(&b.0));
        if satellite.is_empty() {
            return Err(Status::not_found("No satellite found"));
        }
        let satellite = satellite[0].1.clone();
        let satellite = Satellite::from_with_config(satellite, &self.cfg.pox).map_err(|e| {
            Status::internal(format!("Error converting Satellite: {}", e.to_string()))
        })?;
        let zkp = ZKT {};

        let pox = pox::PoX::new(satellite, zkp, &self.cfg.pox)
            .map_err(|e| Status::internal(format!("Error creating PoX: {}", e.to_string())))?;
        let pod = pox.eval_pod();
        let pof = pox.eval_pof();
        let before_compress_pof = bincode::serialize(&pof)
            .map_err(|e| Status::internal(format!("Error serializing PoF: {}", e.to_string())))?;
        let before_compress_pod = bincode::serialize(&pod)
            .map_err(|e| Status::internal(format!("Error serializing PoD: {}", e.to_string())))?;

        let mut pod_encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        let mut pof_encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        bincode::serialize_into(&mut pod_encoder, &pod)
            .map_err(|e| Status::internal(format!("Error serializing PoD: {}", e.to_string())))?;
        bincode::serialize_into(&mut pof_encoder, &pof)
            .map_err(|e| Status::internal(format!("Error serializing PoD: {}", e.to_string())))?;

        let pod_s = pod_encoder
            .finish()
            .map_err(|e| Status::internal(format!("Error Compress PoD: {}", e.to_string())))?;
        let pof_s = pof_encoder
            .finish()
            .map_err(|e| Status::internal(format!("Error Compress PoF: {}", e.to_string())))?;
        info!(
            "PoD compress : {:?} => {:?} PoF compress {:?} => {:?}",
            before_compress_pod.len(),
            pod_s.len(),
            before_compress_pof.len(),
            pof_s.len(),
        );
        info!("{:?}", pod.value);
        info!("{:?}", pof.value);

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
    async fn verify_proof(
        &self,
        request: Request<ZkVerifyProofRequest>,
    ) -> Result<Response<ZkVerifyProofResponse>, Status> {
        let zk_request = request.into_inner();
        debug!("Received request: {:?}", zk_request);
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
        let pod_s = ZlibDecoder::new(&pod_s[..]);
        let pof_s = ZlibDecoder::new(&pof_s[..]);

        let pod: pox::PoDSatelliteResult<BigInt> = bincode::deserialize_from(pod_s)
            .map_err(|e| Status::internal(format!("Error deserializing PoD: {}", e.to_string())))?;
        if !pod.verify() {
            return Ok(Response::new(ZkVerifyProofResponse { is_valid: false }));
        }
        let pof: pox::PoFSatelliteResult<BigInt> = bincode::deserialize_from(pof_s)
            .map_err(|e| Status::internal(format!("Error deserializing PoF: {}", e.to_string())))?;
        let verf = pof.verify();
        let pof_verf = verf.iter().all(|x| *x == pox::PoFVerify::Success);
        verf.iter().for_each(|r| match r {
            pox::PoFVerify::Success => info!("{:?}", r),
            pox::PoFVerify::Fail(f) => error!("{:?}", f),
        });

        let response = ZkVerifyProofResponse { is_valid: pof_verf };
        Ok(Response::new(response))
    }
}
impl ZkRpcServer {
    pub async fn new(cfg: &Config) -> Result<Self, anyhow::Error> {
        Ok(Self {
            addr: format!("{}:{}", cfg.rpc.rpc_host, cfg.rpc.rpc_port),
            db: MockLocalDB::new(&cfg.da_layer).await?,
            cfg: cfg.clone(),
        })
    }
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("zkRpcServer listening on {}", self.addr);
        let t: ZkRpcServer = self.clone();
        Server::builder()
            .add_service(pb::zk_service_server::ZkServiceServer::new(t))
            .serve(self.addr.parse()?)
            .await?;
        Ok(())
    }
}
