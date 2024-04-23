pub mod pb {
    tonic::include_proto!("grpc.zkrpc.service");
}
use pb::{ZkRequest, ZkResponse};
use tonic::{transport::Server, Request, Response, Status};
#[derive(Default)]
pub struct ZkRpcServer {}

#[tonic::async_trait]
impl pb::zk_service_server::ZkService for ZkRpcServer {
    async fn get_proof(&self, request: Request<ZkRequest>) -> Result<Response<ZkResponse>, Status> {
        let resp = pb::ZkResponse {
            address: request.get_ref().address.clone(),
            block_height: request.get_ref().block_height,
            epoch: request.get_ref().epoch,
            alpha_proof: "alpha proof".to_string(),
            beta_proof: "beta proof".to_string(),
        };
        Ok(Response::new(resp))
    }
}
impl ZkRpcServer {
    pub async fn start() -> anyhow::Result<()> {
        let addr = "[::1]:12345".parse().unwrap();
        let zkserver = ZkRpcServer::default();

        println!("zkRpcServer listening on {}", addr);

        Server::builder()
            .add_service(pb::zk_service_server::ZkServiceServer::new(zkserver))
            .serve(addr)
            .await?;
        Ok(())
    }
}
