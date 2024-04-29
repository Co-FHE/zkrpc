mod rpc;
use anyhow::Result;
use logger::initialize_logger;
use rpc::ZkRpcServer;
#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::config::Config::new()?;
    let _guard = initialize_logger(&cfg.log);
    let rpc_server = ZkRpcServer::new(&cfg).await?;
    rpc_server.start().await?;
    Ok(())
}
