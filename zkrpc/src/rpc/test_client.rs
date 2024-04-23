pub mod pb {
    tonic::include_proto!("grpc.zkrpc.service");
}
#[cfg(test)]
mod test_client {
    use crate::rpc::zkrpc;

    use super::*;
    use pb::zk_service_client::ZkServiceClient;
    use pb::ZkRequest;
    use tokio::runtime::Runtime;

    #[test]
    fn test_zk_rpc() {
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            // Start the server
            let _server_handle = tokio::spawn(async { zkrpc::ZkRpcServer::start().await.unwrap() });

            let addr: &str = "http://[::1]:12345";
            let mut client = ZkServiceClient::connect(addr).await.unwrap();
            // define mock request
            let address_mock = "0x123456";
            let block_height_mock = 1;
            let epoch_mock = 2;
            let request = tonic::Request::new(ZkRequest {
                address: address_mock.into(),
                block_height: block_height_mock,
                epoch: epoch_mock,
            });
            let response = client.get_proof(request).await;
            assert!(response.is_ok(), "Expected Ok response, got {:?}", response);

            let unwrapped_response = response.unwrap().into_inner();
            assert!(
                unwrapped_response.address == address_mock,
                "Expected address to be {}, got {}",
                address_mock,
                unwrapped_response.address
            );
            assert!(
                unwrapped_response.block_height == block_height_mock,
                "Expected block height to be {}, got {}",
                block_height_mock,
                unwrapped_response.block_height
            );
            assert!(
                unwrapped_response.epoch == epoch_mock,
                "Expected epoch to be {}, got {}",
                epoch_mock,
                unwrapped_response.epoch
            );
        });
    }
}
