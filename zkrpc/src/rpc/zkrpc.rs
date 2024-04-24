pub mod pb {
    tonic::include_proto!("grpc.zkrpc.service");
}
use config::config::RpcConfig;
use pb::*;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{debug, info};
#[derive(Debug, Clone)]
pub struct ZkRpcServer {
    pub addr: String,
}

#[tonic::async_trait]
impl pb::zk_service_server::ZkService for ZkRpcServer {
    async fn gen_proof(
        &self,
        request: Request<ZkGenProofRequest>,
    ) -> Result<Response<ZkGenProofResponse>, Status> {
        let zk_request = request.into_inner();
        debug!("Received request: {:?}", zk_request);

        let response = ZkGenProofResponse {
            alpha_proof_merkle_root: "alpha_proof_merkle_root".to_string(),
            beta_proof_merkle_root: "beta_proof_merkle_root".to_string(),
            terminal_weights: Default::default(),
        };
        Ok(Response::new(response))
    }
    async fn verify_proof(
        &self,
        request: Request<ZkVerifyProofRequest>,
    ) -> Result<Response<ZkVerifyProofResponse>, Status> {
        let zk_request = request.into_inner();
        debug!("Received request: {:?}", zk_request);

        let response = ZkVerifyProofResponse { is_valid: true };
        Ok(Response::new(response))
    }
}
impl ZkRpcServer {
    pub fn new(cfg: &RpcConfig) -> Self {
        Self {
            addr: format!("{}:{}", cfg.rpc_host, cfg.rpc_port),
        }
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
