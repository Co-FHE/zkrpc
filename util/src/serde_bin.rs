use crate::compressor::{
    BrotliCompressor, CompressorKind, CompressorTrait, Flate2Compressor, RawCompressor,
};
use config::CompressorConfig;
use std::{io, time::Instant};
use tracing::{debug, debug_span};
pub trait SerdeBinTrait: Sized + serde::Serialize + serde::de::DeserializeOwned {
    fn serialize_compress<C: CompressorTrait>(
        &self,
        cfg: &CompressorConfig,
    ) -> color_eyre::Result<Vec<u8>> {
        let _span = debug_span!("serialize_compress").entered();
        let start_time = Instant::now();
        let data = bincode::serialize(&self)?;
        let compressor = C::new(cfg);
        //concat compressor.kind() and compressed_data
        let mut compressed_data = compressor.compress(&data)?;
        compressed_data.insert(0, compressor.kind() as u8);

        debug!(
            rate = %format!(
                "{{{} => {}}}({:.2}%)",
                data.len(),
                compressed_data.len(),
                compressed_data.len() as f64 / data.len() as f64 * 100.0
            ),
            compression_time = ?start_time.elapsed(),
            compressor = ?compressor,
        );
        Ok(compressed_data)
    }
    fn decompress_deserialize(data: &Vec<u8>, cfg: &CompressorConfig) -> color_eyre::Result<Self> {
        let _span = debug_span!("decompress_deserialize").entered();
        let start_time = Instant::now();
        if data.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "data is empty").into());
        }
        let kind = CompressorKind::try_from(data[0])?;
        let data = &data[1..].to_vec();
        let (de_data, debug_info) = match kind {
            CompressorKind::Raw => {
                let decompressor = RawCompressor::new(cfg);
                (
                    decompressor.decompress(data)?,
                    format!("{:?}", decompressor),
                )
            }
            CompressorKind::Brotli => {
                let decompressor = BrotliCompressor::new(cfg);
                (
                    decompressor.decompress(data)?,
                    format!("{:?}", decompressor),
                )
            }
            CompressorKind::Flate2 => {
                let decompressor = Flate2Compressor::new(cfg);
                (
                    decompressor.decompress(data)?,
                    format!("{:?}", decompressor),
                )
            }
        };
        let decompressed_data = bincode::deserialize(&de_data)?;
        debug!(
            rate = %format!(
                "{{{} => {}}}({:.2}%)",
                data.len(),
                de_data.len(),
                data.len() as f64 / de_data.len() as f64 * 100.0
            ),
            decompression_time = ?start_time.elapsed(),
            decompressor = %debug_info,
        );

        Ok(decompressed_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::compressor::BrotliCompressor;

    use super::*;
    use config::CompressorConfig;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        a: u32,
        b: String,
    }

    impl SerdeBinTrait for TestStruct {}

    #[test]
    fn test_serde_bin_trait() {
        let _guard = logger::init_logger_for_test!();
        let cfg = CompressorConfig::default();
        let test_struct = TestStruct {
            a: 1,
            b: "test".to_owned(),
        };
        let compressed_data = test_struct
            .serialize_compress::<BrotliCompressor>(&cfg)
            .unwrap();
        let decompressed_data = TestStruct::decompress_deserialize(&compressed_data, &cfg).unwrap();
        assert_eq!(test_struct, decompressed_data);

        let compressed_data = test_struct
            .serialize_compress::<RawCompressor>(&cfg)
            .unwrap();
        let decompressed_data = TestStruct::decompress_deserialize(&compressed_data, &cfg).unwrap();
        assert_eq!(test_struct, decompressed_data);

        let compressed_data = test_struct
            .serialize_compress::<Flate2Compressor>(&cfg)
            .unwrap();
        let decompressed_data = TestStruct::decompress_deserialize(&compressed_data, &cfg).unwrap();
        assert_eq!(test_struct, decompressed_data);
    }
}
