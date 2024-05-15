use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct RpcConfig {
    pub rpc_port: u16,
    pub rpc_host: String,
    pub client_host: String,
    pub timeout: u64,
    pub enable_mesh_rpc: bool,
}
impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            rpc_port: 15937,
            rpc_host: "127.0.0.1".to_owned(),
            client_host: "127.0.0.1".to_owned(),
            timeout: 60,
            enable_mesh_rpc: true,
        }
    }
}
