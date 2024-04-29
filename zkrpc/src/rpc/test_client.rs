pub mod pb {
    tonic::include_proto!("grpc.zkrpc.service");
}
#[cfg(test)]
mod test_client {
    use std::path::PathBuf;

    use crate::rpc::zkrpc;

    use super::*;
    use logger::initialize_logger;
    use pb::zk_service_client::ZkServiceClient;
    use pb::*;
    use tokio::runtime::Runtime;
    use tracing::{error, info};
    #[test]
    fn test_zg(){
        
    }
    #[test]
    fn test_zk_rpc() {
        let _guard = initialize_logger(&config::config::LogConfig {
            log_level: config::config::LogLevel::Info,
            write_to_file: false,
            show_file_path: false,
            show_line_number: false,
            show_with_target: false,
            log_dir: PathBuf::new(),
            rotation: config::config::LogRotation::Never,
        });
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            // Start the server
            let port = 15937;
            // let _server_handle = tokio::spawn(async move {
            //     let mut cfg = config::config::Config::new().unwrap();
            //     cfg.rpc = config::config::RpcConfig {
            //         rpc_host: "127.0.0.1".to_string(),
            //         rpc_port: port,
            //     };
            //     zkrpc::ZkRpcServer::new(&cfg)
            //         .await
            //         .unwrap()
            //         .start()
            //         .await
            //         .unwrap();
            // });

            let mut client = ZkServiceClient::connect(format!("http://127.0.0.1:{}", port))
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
        });
    }
}
