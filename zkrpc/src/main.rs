mod rpc;
use anyhow::Result;
use rpc::ZkRpcServer;
#[tokio::main]
async fn main() -> Result<()> {
    let _ = ZkRpcServer::start().await?;
    Ok(())
}
