#[cfg(test)]
mod tests {

    use crate::bootstrap::client::bootstrap_identify_registry;
    use crate::bootstrap::server::start_bootstrap_server;
    //cargo test --package p2p --lib -- bootstrap::tests::tests::test_server --exact --show-output
    #[tokio::test]
    async fn test_server() {
        let _guard = logger::init_logger_for_test!();
        let cfg = config::P2PConfig::default();
        start_bootstrap_server(cfg.bootstrap_config).await.unwrap(); // Pass cfg.bootstrap_config as a parameter
    }

    //cargo test --package p2p --lib -- bootstrap::tests::tests::test_client --exact --show-output
    #[tokio::test]
    async fn test_client() {
        let _guard = logger::init_logger_for_test!();
        let cfg = config::P2PConfig::default();
        bootstrap_identify_registry(&cfg.bootstrap_config).await;
    }
}
