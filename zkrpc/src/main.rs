mod rpc;
use anyhow::Result;
use config::config::{LogConfig, LogLevel};
use logger::initialize_logger;
use pb::*;
use rpc::{pb, ZkRpcServer};

use clap::ArgAction;
use clap::{Args, Parser, Subcommand};
use pb::zk_service_client::ZkServiceClient;
use tracing::{error, info};

#[derive(Subcommand)]
pub enum Commands {
    /// list all passwords
    Client,
    Server,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    pub command: Commands,
}
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Server => {
            let mut cfg = config::config::Config::new()?;
            cfg.log.log_level = LogLevel::Info;
            let _guard = initialize_logger(&cfg.log);
            let rpc_server = ZkRpcServer::new(&cfg).await?;
            rpc_server.start().await?;
            Ok(())
        }
        Commands::Client => {
            let mut cfg = config::config::Config::new()?;
            cfg.log.log_level = LogLevel::Info;
            let _guard = initialize_logger(&cfg.log);
            let mut client = ZkServiceClient::connect(format!(
                "http://{}:{}",
                cfg.rpc.client_host, cfg.rpc.rpc_port
            ))
            .await
            .unwrap();
            let mut start = 200000;
            loop {
                info!("start: {:?}", start);
                // define mock request
                let prover_address_mock = "0x123456";
                let satellite_address_mock = "evmosvaloper1q9dvfsksdv88yz8yjzm6xy808888ylc8e2n838";
                let epoch_for_proof_mock = 1;
                let block_height_from_for_proof_mock = start;
                let block_height_to_for_proof_mock = start + 10;
                let request = tonic::Request::new(ZkGenProofRequest {
                    prover_address: prover_address_mock.to_string(),
                    satellite_address: satellite_address_mock.to_string(),
                    epoch_for_proof: epoch_for_proof_mock,
                    block_height_from_for_proof: block_height_from_for_proof_mock,
                    block_height_to_for_proof: block_height_to_for_proof_mock,
                });
                let response = client.gen_proof(request).await;
                info!("response: {:?}", response);
                if response.is_err() {
                    error!("response: {:?}", response);
                }
                // assert!(response.is_ok(), "Expected Ok response, got {:?}", response);
                // inner into the response
                if let Err(e) = response {
                    error!("response: {:?}", e);
                    continue;
                }
                let resp_unwrapped = response.unwrap().into_inner();
                let request = tonic::Request::new(ZkVerifyProofRequest {
                    prover_address: prover_address_mock.to_string(),
                    satellite_address: satellite_address_mock.to_string(),
                    epoch_for_proof: epoch_for_proof_mock,
                    block_height_from_for_proof: block_height_from_for_proof_mock,
                    block_height_to_for_proof: block_height_to_for_proof_mock,
                    alpha_proof_merkle_root: resp_unwrapped.alpha_proof_merkle_root,
                    beta_proof_merkle_root: resp_unwrapped.beta_proof_merkle_root,
                    // terminal_weights: resp_unwrapped.terminal_weights,
                });
                let response = client.verify_proof(request).await;
                if response.is_err() {
                    error!("response: {:?}", response);
                }
                // assert!(response.is_ok(), "Expected Ok response, got {:?}", response);
                if let Err(e) = response {
                    error!("response: {:?}", e);
                    continue;
                }
                let resp_unwrapped = response.unwrap().into_inner();
                info!("response: {:?}", resp_unwrapped);
                // assert!(
                //     resp_unwrapped.is_valid,
                //     "Expected valid proof, got {:?}",
                //     resp_unwrapped
                // );
                start += 10;
            }
        }
    }
}
