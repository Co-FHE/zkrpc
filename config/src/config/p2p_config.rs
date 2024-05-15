use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct P2PConfig {
    pub bootstrap_config: BootstrapConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct BootstrapConfig {
    pub bootstrap_port: u16,
    pub fixed_seed: Option<[u8; 32]>, // if set, use this as the seed for the random number generator, otherwise generate a random seed
    pub enable_lan: bool,
    pub register_timeout: u64,
    pub register_retry_interval: u64,
    pub register_port: u16,
}
impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            bootstrap_port: 31488,
            fixed_seed: Some([
                68, 114, 46, 32, 83, 104, 105, 111, 116, 111, 108, 105, 32, 103, 101, 110, 101,
                114, 97, 116, 101, 100, 32, 116, 104, 101, 32, 115, 101, 101, 100, 46,
            ]),
            enable_lan: false,
            register_timeout: 10,
            register_retry_interval: 10,
            register_port: 31489,
        }
    }
}
impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            bootstrap_config: BootstrapConfig::default(),
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn generate_random_seed() {
        use rand::Rng;
        let r = "Dr. Shiotoli generated the seed.".as_bytes();
        println!("{:?}", r);
        let mut rng = rand::thread_rng();
        let mut arr: [u8; 32] = [0; 32];
        for i in arr.iter_mut() {
            *i = rng.gen();
        }
        println!("{:?}", arr);
    }
}
