use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcConfig {
    pub rpc_port: u16,
    pub rpc_host: String,
    pub client_host: String,
}
impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            rpc_port: 15937,
            rpc_host: "127.0.0.1".to_owned(),
            client_host: "127.0.0.1".to_owned(),
        }
    }
}
