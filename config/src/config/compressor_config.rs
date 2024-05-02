use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct CompressorConfig {
    pub brotli: BrotliConfig,
    pub flate2: Flate2Config,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct BrotliConfig {
    pub quality: u32,
    pub lgwin: u32,
    pub buffer_size: usize,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub enum Flate2CompressorType {
    Gzip,
    Zlib,
    Deflate,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct Flate2Config {
    pub level: u32,
    pub flate2_type: Flate2CompressorType,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            brotli: BrotliConfig::default(),
            flate2: Flate2Config::default(),
        }
    }
}
impl Default for BrotliConfig {
    fn default() -> Self {
        Self {
            quality: 11,
            lgwin: 20,
            buffer_size: 4096,
        }
    }
}

impl Default for Flate2Config {
    fn default() -> Self {
        Self {
            level: 9,
            flate2_type: Flate2CompressorType::Zlib,
        }
    }
}
