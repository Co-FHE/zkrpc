mod rpc;
use anyhow::Result;
use config::config::Config;
use logger::initialize_logger;
use rpc::ZkRpcServer;
#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::config::Config::new()?;
    let _guard = initialize_logger(&cfg.log);
    let rpc_server = ZkRpcServer::new(&cfg.rpc);
    rpc_server.start().await?;
    Ok(())
}
