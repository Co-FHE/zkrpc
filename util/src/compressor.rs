use std::io::{self, Read, Write};

use config::{CompressorConfig, Flate2CompressorType};
use flate2::{
    read::{DeflateDecoder, GzDecoder, ZlibDecoder},
    write::{DeflateEncoder, GzEncoder, ZlibEncoder},
    Compression,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrotliCompressor {
    buffer_len: usize,
    lgwin: u32,
    quality: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flate2Compressor {
    level: u32,
    flate2_type: Flate2CompressorType,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawCompressor;
pub enum CompressorKind {
    Raw = 0,
    Brotli = 1,
    Flate2 = 2,
}
impl Into<u8> for CompressorKind {
    fn into(self) -> u8 {
        self as u8
    }
}
impl TryFrom<u8> for CompressorKind {
    type Error = io::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CompressorKind::Raw),
            1 => Ok(CompressorKind::Brotli),
            2 => Ok(CompressorKind::Flate2),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid compressor kind {}", value),
            )),
        }
    }
}
pub trait CompressorTrait: std::fmt::Debug + std::marker::Sync + Eq + PartialEq {
    fn new(compressor_config: &CompressorConfig) -> Self;
    fn compress(&self, data: &Vec<u8>) -> io::Result<Vec<u8>>;
    fn decompress(&self, data: &Vec<u8>) -> io::Result<Vec<u8>>;
    fn kind(&self) -> CompressorKind;
}
impl CompressorTrait for RawCompressor {
    fn new(_compressor_config: &CompressorConfig) -> Self {
        Self
    }
    fn compress(&self, data: &Vec<u8>) -> io::Result<Vec<u8>> {
        Ok(data.to_owned())
    }
    fn decompress(&self, data: &Vec<u8>) -> io::Result<Vec<u8>> {
        Ok(data.to_owned())
    }

    fn kind(&self) -> CompressorKind {
        CompressorKind::Raw
    }
}
impl CompressorTrait for BrotliCompressor {
    fn new(compressor_config: &CompressorConfig) -> Self {
        Self {
            buffer_len: compressor_config.brotli.buffer_size,
            lgwin: compressor_config.brotli.lgwin,
            quality: compressor_config.brotli.quality,
        }
    }
    fn compress(&self, data: &Vec<u8>) -> io::Result<Vec<u8>> {
        let mut compressed = Vec::new();
        let mut compressor =
            brotli::CompressorReader::new(&data[..], self.buffer_len, self.quality, self.lgwin);
        compressor.read_to_end(&mut compressed)?;
        Ok(compressed)
    }

    fn decompress(&self, data: &Vec<u8>) -> io::Result<Vec<u8>> {
        let mut decompressed = Vec::new();
        let mut decompressor = brotli::Decompressor::new(&data[..], self.buffer_len);
        decompressor.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn kind(&self) -> CompressorKind {
        CompressorKind::Brotli
    }
}
impl CompressorTrait for Flate2Compressor {
    fn new(compressor_config: &CompressorConfig) -> Self {
        Self {
            level: compressor_config.flate2.level,
            flate2_type: compressor_config.flate2.flate2_type.to_owned(),
        }
    }

    fn compress(&self, data: &Vec<u8>) -> io::Result<Vec<u8>> {
        match self.flate2_type {
            Flate2CompressorType::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.level));
                encoder.write_all(&data)?;
                encoder.finish()
            }
            Flate2CompressorType::Zlib => {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(self.level));
                encoder.write_all(&data)?;
                encoder.finish()
            }
            Flate2CompressorType::Deflate => {
                let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(self.level));
                encoder.write_all(&data)?;
                encoder.finish()
            }
        }
    }

    fn decompress(&self, compressed_data: &Vec<u8>) -> io::Result<Vec<u8>> {
        match self.flate2_type {
            Flate2CompressorType::Gzip => {
                let mut decoder = GzDecoder::new(&compressed_data[..]);
                let mut decompressed_data = Vec::new();
                decoder.read_to_end(&mut decompressed_data)?;

                Ok(decompressed_data)
            }
            Flate2CompressorType::Zlib => {
                let mut decoder = ZlibDecoder::new(&compressed_data[..]);
                let mut decompressed_data = Vec::new();
                decoder.read_to_end(&mut decompressed_data)?;

                Ok(decompressed_data)
            }
            Flate2CompressorType::Deflate => {
                let mut decoder = DeflateDecoder::new(&compressed_data[..]);
                let mut decompressed_data = Vec::new();
                decoder.read_to_end(&mut decompressed_data)?;

                Ok(decompressed_data)
            }
        }
    }

    fn kind(&self) -> CompressorKind {
        CompressorKind::Flate2
    }
}

#[cfg(test)]
mod tests {

    use std::time::Instant;

    use super::*;
    use config::{BrotliConfig, CompressorConfig, Flate2Config};
    use logger::init_logger_for_test;
    use tracing::{info, span, Level};
    fn test_specific_compressor<T: CompressorTrait>(compressor: T, data: &Vec<u8>) {
        let span = span!(Level::INFO, "compression test");
        let _enter = span.enter();

        let start = Instant::now();

        let compressed = compressor.compress(&data).unwrap();
        let compression_time = start.elapsed();
        let decompressed = compressor.decompress(&compressed).unwrap();
        let decompression_time = start.elapsed() - compression_time;
        assert_eq!(data.to_owned(), decompressed);
        info!(
            compressor= ?compressor,
            ?compression_time,
            ?decompression_time,
            "data len: {} => {}",
            data.len(),
            compressed.len(),
        );
    }
    #[test]
    fn test_compressor() {
        let _guard = init_logger_for_test!();
        let mut compressor_config = CompressorConfig {
            brotli: BrotliConfig {
                quality: 11,
                lgwin: 22,
                buffer_size: 4096,
            },
            flate2: Flate2Config {
                level: 9,
                flate2_type: Flate2CompressorType::Zlib,
            },
        };
        let data = include_bytes!("../../testdata/pi_big.bin").to_vec();

        test_specific_compressor(BrotliCompressor::new(&compressor_config), &data);
        compressor_config.brotli.buffer_size = 1024;
        test_specific_compressor(BrotliCompressor::new(&compressor_config), &data);
        compressor_config.brotli.buffer_size = 4096;
        compressor_config.brotli.lgwin = 20;
        test_specific_compressor(BrotliCompressor::new(&compressor_config), &data);

        compressor_config.flate2.flate2_type = Flate2CompressorType::Zlib;
        test_specific_compressor(Flate2Compressor::new(&compressor_config), &data);
        compressor_config.flate2.flate2_type = Flate2CompressorType::Gzip;
        test_specific_compressor(Flate2Compressor::new(&compressor_config), &data);
        compressor_config.flate2.flate2_type = Flate2CompressorType::Deflate;
        test_specific_compressor(Flate2Compressor::new(&compressor_config), &data);
    }
}
