use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

pub mod pb {
    tonic::include_proto!("grpc.zkrpc.service");
}
#[cfg(test)]
mod tests {
    use super::*;
    use pb::zk_service_client::ZkServiceClient;
    use pb::ZkRequest;
    use tokio::runtime::Runtime;
    use tonic::transport::Channel;

    #[test]
    fn test_zk_rpc() {
        // Create a runtime for the async client
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            // Set the address for the service
            let addr = "http://[::1]:50051".parse().unwrap();

            // Create the client
            let mut client = ZkServiceClient::connect(addr).await.unwrap();

            // Create a request
            let request = tonic::Request::new(ZkRequest {
                // populate fields as needed
            });

            // Send the request
            let response = client.get_proof(request).await;

            // Verify the response
            assert!(response.is_ok(), "Expected Ok response, got {:?}", response);

            let unwrapped_response = response.unwrap();
            assert!(
                !unwrapped_response.into_inner().alpha_proof.is_empty(),
                "Expected non-empty alpha proof"
            );
            assert!(
                !unwrapped_response.into_inner().beta_proof.is_empty(),
                "Expected non-empty beta proof"
            );
        });
    }
}
