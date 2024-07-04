pub mod pb {
    tonic::include_proto!("grpc.zkrpc.service");
}
#[cfg(test)]
mod test_client {

    use std::thread;
    use std::time::Duration;

    use crate::rpc::zkrpc;

    use super::*;
    use config::LogLevel;
    use logger::initialize_logger;
    use pb::zk_service_client::ZkServiceClient;
    use pb::*;
    use tokio::runtime::Runtime;
    use tracing::info;
    #[test]
    fn test_zk_rpc_timeout() {
        let mut cfg = config::Config::new().unwrap();
        cfg.log.log_level = LogLevel::Info;
        let _guard = initialize_logger(&cfg.log);
        let rt = Runtime::new().unwrap();
        let mut cfg = config::Config::new().unwrap();
        cfg.rpc = config::RpcConfig {
            rpc_host: "127.0.0.1".to_string(),
            rpc_port: 57392,
            client_host: "127.0.0.1".to_string(),
            timeout: 0,
            enable_mesh_rpc: false,
        };
        rt.block_on(async {
            // Start the server
            let port = 57392;
            let _server_handle = tokio::spawn(async move {
                zkrpc::ZkRpcServer::new(&cfg)
                    .await
                    .unwrap()
                    .start()
                    .await
                    .unwrap();
            });
            thread::sleep(Duration::from_secs(10));

            let mut client = ZkServiceClient::connect(format!("http://127.0.0.1:{}", port))
                .await
                .unwrap();
            let start = 0;
            // define mock request
            let prover_address_mock = "0x123456";
            let remote_address_mock = "evmosvaloper1q9dvfsksdv88yz8yjzm6xy808888ylc8e2n838";
            let epoch_for_proof_mock = 1;
            let block_height_from_for_proof_mock = start;
            let block_height_to_for_proof_mock = start + 200000;
            let request = tonic::Request::new(ZkGenProofRequest {
                prover_address: prover_address_mock.to_string(),
                remote_address: remote_address_mock.to_string(),
                epoch_for_proof: epoch_for_proof_mock,
                block_height_from_for_proof: block_height_from_for_proof_mock,
                block_height_to_for_proof: block_height_to_for_proof_mock,
            });
            let response = client.gen_proof(request).await;
            info!("response: {:?}", response);
            // assert!(response.is_ok(), "Expected Ok response, got {:?}", response);
            // inner into the response
            if let Err(e) = response {
                match e.code() {
                    tonic::Code::DeadlineExceeded => {
                        info!("Expected DeadlineExceeded response, got {:?}", e);
                    }
                    _ => {
                        panic!("response: {:?}", e);
                    }
                }
            }
        });
    }
    #[test]
    fn test_zk_rpc() {
        let mut cfg = config::Config::new().unwrap();
        cfg.log.log_level = LogLevel::Info;
        let _guard = initialize_logger(&cfg.log);
        let rt = Runtime::new().unwrap();
        let mut cfg = config::Config::new().unwrap();
        cfg.rpc = config::RpcConfig {
            rpc_host: "127.0.0.1".to_string(),
            rpc_port: 57398,
            client_host: "127.0.0.1".to_string(),
            timeout: 1000,
            enable_mesh_rpc: false,
        };
        rt.block_on(async {
            // Start the server
            let port = 57398;
            let _server_handle = tokio::spawn(async move {
                zkrpc::ZkRpcServer::new(&cfg)
                    .await
                    .unwrap()
                    .start()
                    .await
                    .unwrap();
            });
            thread::sleep(Duration::from_secs(10));

            let mut client = ZkServiceClient::connect(format!("http://127.0.0.1:{}", port))
                .await
                .unwrap();
            let start = 0;
            // define mock request
            let prover_address_mock = "0x123456";
            let remote_address_mock = "space1fdhkvj4zjgverz2fsy6cmehxx6gtxrwh0j7pch";
            let epoch_for_proof_mock = 1;
            let block_height_from_for_proof_mock = start;
            let block_height_to_for_proof_mock = start + 200000;
            let request = tonic::Request::new(ZkGenProofRequest {
                prover_address: prover_address_mock.to_string(),
                remote_address: remote_address_mock.to_string(),
                epoch_for_proof: epoch_for_proof_mock,
                block_height_from_for_proof: block_height_from_for_proof_mock,
                block_height_to_for_proof: block_height_to_for_proof_mock,
            });
            let response = client.gen_proof(request).await;
            info!("response: {:?}", response);
            if response.is_err() {
                panic!("response: {:?}", response);
            }
            // assert!(response.is_ok(), "Expected Ok response, got {:?}", response);
            // inner into the response
            if let Err(e) = response {
                panic!("response: {:?}", e);
            }
            let resp_unwrapped = response.unwrap().into_inner();
            let request = tonic::Request::new(ZkVerifyProofRequest {
                prover_address: prover_address_mock.to_string(),
                remote_address: remote_address_mock.to_string(),
                epoch_for_proof: epoch_for_proof_mock,
                block_height_from_for_proof: block_height_from_for_proof_mock,
                block_height_to_for_proof: block_height_to_for_proof_mock,
                alpha_proof_merkle_root: resp_unwrapped.alpha_proof_merkle_root,
                beta_proof_merkle_root: resp_unwrapped.beta_proof_merkle_root,
                // terminal_weights: resp_unwrapped.terminal_weights,
            });
            let response = client.verify_proof(request).await;
            if response.is_err() {
                panic!("response: {:?}", response);
            }
            // assert!(response.is_ok(), "Expected Ok response, got {:?}", response);
            if let Err(e) = response {
                panic!("response: {:?}", e);
            }
            let resp_unwrapped = response.unwrap().into_inner();
            assert!(resp_unwrapped.is_valid);
            // assert!(
            //     resp_unwrapped.is_valid,
            //     "Expected valid proof, got {:?}",
            //     resp_unwrapped
            // );
        });
    }
}
