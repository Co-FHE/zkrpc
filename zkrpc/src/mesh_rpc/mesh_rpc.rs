use std::{
    sync::Arc,
    time::{self, SystemTime},
};

use config::{BaseConfig, Config, BASE_CONFIG};
use da_layer::MockLocalDB;
use tokio::{fs, sync::Mutex};
use tonic::{Request, Response, Status};
use tracing::{info, info_span, warn, Instrument};

use self::mesh_pb::*;

pub mod mesh_pb {
    tonic::include_proto!("grpc.mesh.service");
}

#[derive(Debug)]
pub struct MeshRpcServer {
    pub addr: String,
    pub private_key: Mutex<Arc<Option<String>>>,
    pub cfg: Config,
}
impl MeshRpcServer {
    pub async fn new(cfg: &Config) -> color_eyre::Result<Self> {
        let path = BASE_CONFIG.root_path.join("keystore/mesh_blockchain.key");
        let addr = format!("{}:{}", cfg.rpc.rpc_host, cfg.rpc.rpc_port);
        // if path.exists() , backup the file and  write the private_key to the file
        if path.exists() {
            // read string from file
            let private_key = fs::read_to_string(&path).await?;
            info!("find private key from file, path: {:?}", path);
            Ok(Self {
                addr,
                private_key: Mutex::new(Arc::new(Some(private_key))),
                cfg: cfg.clone(),
            })
        } else {
            Ok(Self {
                addr,
                private_key: Mutex::new(Arc::new(None)),
                cfg: cfg.clone(),
            })
        }

        // Ok(Self {
        //     private_key: private_key,
        //     cfg: cfg.clone(),
        // })
    }
    pub async fn start(self) -> color_eyre::Result<()> {
        // let t: MeshRpcServer = self.clone();
        let addr = self.addr.clone();
        tonic::transport::Server::builder()
            .add_service(mesh_pb::mesh_service_server::MeshServiceServer::new(self))
            .serve(addr.parse()?)
            .await?;
        Ok(())
    }
}
#[tonic::async_trait]
impl mesh_pb::mesh_service_server::MeshService for MeshRpcServer {
    async fn init(
        &self,
        request: Request<InitMeshRequest>,
    ) -> Result<Response<InitMeshResponse>, Status> {
        let ip = request
            .remote_addr()
            .map_or("unknow".to_string(), |addr| addr.ip().to_string());
        let req = request.into_inner();
        async {
            let path = BASE_CONFIG.root_path.join("keystore/mesh_blockchain.key");
            if path.exists() {
                let backup_path =
                    BASE_CONFIG
                        .root_path
                        .join(format!("keystore/mesh_blockchain.key.{:?}.bak", {
                            let now = SystemTime::now();

                            match now.duration_since(SystemTime::UNIX_EPOCH) {
                                Ok(n) => n.as_secs() as i64,
                                Err(e) => {
                                    warn!("time error, backup file by -1, error: {}", e);
                                    -1
                                }
                            }
                        }));
                info!(
                    "path {} exists, backup the file to {}",
                    path.display(),
                    backup_path.display()
                );
                std::fs::rename(&path, &backup_path)?;
            }
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            std::fs::write(&path, req.private_key.clone())?;
            *self.private_key.lock().await = Arc::new(Some(req.private_key.clone()));
            info!("set private key {} success", req.private_key.clone());
            Ok(Response::new(InitMeshResponse { success: true }))
        }
        .instrument(info_span!(
            "mesh_init",
            private_key = req.private_key.clone(),
            ip,
        ))
        .await
    }
    async fn start_meshee(
        &self,
        request: Request<mesh_pb::MeshingRequest>,
    ) -> Result<Response<mesh_pb::MeshingResponse>, Status> {
        if self.private_key.lock().await.is_none() {
            return Err(Status::permission_denied("private key not found"));
        }
        let req = request.into_inner();
        Err(Status::unimplemented("not implemented"))
    }
    async fn verify_meshee(
        &self,
        request: Request<mesh_pb::VerifyMesheeRequest>,
    ) -> Result<Response<mesh_pb::VerifyMesheeResponse>, Status> {
        if self.private_key.lock().await.is_none() {
            return Err(Status::permission_denied("private key not found"));
        }
        let req = request.into_inner();
        Err(Status::unimplemented("not implemented"))
    }
}
