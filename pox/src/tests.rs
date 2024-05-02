#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use config::*;
    use halo2_proofs::pasta::Fp;
    use logger::init_logger_for_test;
    use num_bigint::BigInt;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use tracing::{debug, info};
    use types::{Alpha, CompletePackets, EndPointFrom, Packet, Pos2D, Satellite};
    use util::{compressor::BrotliCompressor, serde_bin::SerdeBinTrait};
    use zkt::ZkTraitHalo2;

    use crate::{
        PoDSatelliteResult, PoDTerminalResult, PoFSatelliteResult, PoFVerify, PoX, PosTrait,
    };

    struct TestZK {}
    impl ZkTraitHalo2 for TestZK {
        type F = Fp;
        fn gen_proof(
            &self,
            _coefs: Vec<Fp>,
            _x: Vec<Fp>,
        ) -> Result<(Vec<u8>, Vec<u8>), zkt::traits::Error> {
            Ok((Vec::new(), Vec::new()))
        }

        fn verify_proof(_out: Vec<u8>, _proof: Vec<u8>) -> bool {
            true
        }

        fn setup() {}
    }
    #[test]
    fn test_pod() {
        let _guard = init_logger_for_test!();
        use crate::PoX;
        use config::PoxConfig;
        use halo2_proofs::pasta::Fp;
        use num_bigint::ToBigInt;
        use rs_merkle::MerkleProof;
        use rust_decimal::prelude::Zero;
        use types::{Alpha, FixedPoint, FixedPointInteger, GetPos2D, Satellite};
        use zkt::ZkTraitHalo2;
        let cfg = PoxConfig {
            kernel: KernelConfig {
                quadratic: QuadraticConfig {
                    max_dis_sqr: dec!(25),
                },
                gaussian: GaussianConfig {
                    sigma: dec!(3),
                    vanilla: GaussianVanillaConfig { use_coef: false },
                    tylor: GaussianTylorConfig {
                        sigma_range: dec!(5.0),
                    },
                },
            },
            penalty: PenaltyConfig { max_diff: dec!(20) },
            rspr_precision_bigint: 4,
            cooridnate_precision_bigint: 3,
            pod_max_value: dec!(-100),
        };
        let satellite = Satellite::<Decimal> {
            terminals: vec![
                types::Terminal {
                    address: "0x1".to_string(),
                    alpha: Alpha { rspr: dec!(-70) },
                    terminal_packets: None,
                    position: Pos2D {
                        x: dec!(0),
                        y: dec!(0),
                    },
                },
                types::Terminal {
                    address: "0x2".to_string(),
                    alpha: Alpha { rspr: dec!(-80) },
                    terminal_packets: None,
                    position: Pos2D {
                        x: dec!(-1),
                        y: dec!(0),
                    },
                },
                types::Terminal {
                    address: "0x3".to_string(),
                    alpha: Alpha { rspr: dec!(-40) },
                    terminal_packets: None,
                    position: Pos2D {
                        x: dec!(0),
                        y: dec!(2),
                    },
                },
                types::Terminal {
                    address: "0x4".to_string(),
                    alpha: Alpha { rspr: dec!(-60) },
                    terminal_packets: None,
                    position: Pos2D {
                        x: dec!(3),
                        y: dec!(0),
                    },
                },
                types::Terminal {
                    address: "0x5".to_string(),
                    alpha: Alpha { rspr: dec!(-50) },
                    terminal_packets: None,
                    position: Pos2D {
                        x: dec!(0),
                        y: dec!(-4),
                    },
                },
            ],
            satellite_packets: None,
            epoch: 1,
            address: "0x123456".to_string(),
            position: types::Pos3D {
                x: dec!(0),
                y: dec!(0),
                height: dec!(10000),
            },
        };
        let zk = TestZK {};
        let satellite_decimal = satellite.clone();
        let satellite = Satellite::from_with_config(satellite, &cfg).unwrap();
        let ss = satellite_decimal
            .terminals
            .iter()
            .map(|t| {
                let weight: Decimal = satellite_decimal
                    .terminals
                    .iter()
                    .map(|t2| {
                        let rtmp = dec!(25) - t.position.dist_sqr(&t2.position);
                        if rtmp.is_sign_negative() {
                            dec!(0)
                        } else {
                            rtmp
                        }
                    })
                    .sum();
                let total: Decimal = satellite_decimal
                    .terminals
                    .iter()
                    .map(|t2| {
                        let rtmp = dec!(25) - t.position.dist_sqr(&t2.position);
                        let rtmp = if rtmp.fixed_is_negative() {
                            dec!(0)
                        } else {
                            rtmp
                        };
                        rtmp * t2.alpha.rspr.clone()
                    })
                    .sum();
                let exact_value = total / weight;
                let diff = dec!(20) - (t.alpha.rspr.clone() - exact_value).abs();
                let diff = if diff.is_sign_negative() {
                    dec!(0)
                } else {
                    diff
                };
                debug!(
                    "Reference: address = {}, value = {}, weight = {}",
                    t.address, exact_value, diff
                );
                (exact_value, diff)
            })
            .collect::<Vec<_>>();
        let sv = ss.iter().map(|(v, d)| v * d).sum::<Decimal>()
            / ss.iter().map(|(_, d)| d).sum::<Decimal>()
            + dec!(100);
        debug!("Reference: value = {}", sv);
        let required_result = PoDSatelliteResult::<BigInt> {
            value: BigInt::from(384606),
            terminal_results: vec![
                PoDTerminalResult {
                    terminal_address: "0x1".to_string(),
                    weight: BigInt::from(123158),
                    value_for_satellite: BigInt::from(-623157),
                    proof: (Vec::new(), Vec::new()),
                },
                PoDTerminalResult {
                    terminal_address: "0x2".to_string(),
                    weight: BigInt::from(30233),
                    value_for_satellite: BigInt::from(-630232),
                    proof: (Vec::new(), Vec::new()),
                },
                PoDTerminalResult {
                    terminal_address: "0x3".to_string(),
                    weight: BigInt::from(0),
                    value_for_satellite: BigInt::from(-614102),
                    proof: (Vec::new(), Vec::new()),
                },
                PoDTerminalResult {
                    terminal_address: "0x4".to_string(),
                    weight: BigInt::from(183871),
                    value_for_satellite: BigInt::from(-616129),
                    proof: (Vec::new(), Vec::new()),
                },
                PoDTerminalResult {
                    terminal_address: "0x5".to_string(),
                    weight: BigInt::from(100000),
                    value_for_satellite: BigInt::from(-600000),
                    proof: (Vec::new(), Vec::new()),
                },
            ],
        };
        let pox = PoX::new(satellite, zk, &cfg).unwrap();
        assert_eq!(pox.kernel.max_dis_sqr, BigInt::from(25_000_000));
        assert_eq!(pox.penalty.max_diff, BigInt::from(200_000));
        let pod_result = pox.eval_pod();
        assert_eq!(required_result, pod_result);
        assert_eq!(
            PoDSatelliteResult::decompress_deserialize(
                &pod_result
                    .serialize_compress::<BrotliCompressor>(&CompressorConfig::default())
                    .unwrap(),
                &CompressorConfig::default()
            )
            .unwrap(),
            PoDSatelliteResult::decompress_deserialize(
                &required_result
                    .serialize_compress::<BrotliCompressor>(&CompressorConfig::default())
                    .unwrap(),
                &CompressorConfig::default()
            )
            .unwrap()
        );

        // assert_eq!(pod_result.value, BigInt::from(1));
        // assert_eq!(pod_result.terminal_results.len(), 2);
        // let pof_result = pox.eval_pof();
        // assert_eq!(pof_result.value, BigInt::zero());
        // assert_eq!(pof_result.terminal_results.len(), 2);
    }
    #[test]
    fn test_pof() {
        let _guard = init_logger_for_test!();
        let cfg = PoxConfig::default();
        let satellite = Satellite::<Decimal> {
            terminals: vec![
                types::Terminal {
                    address: "0x1".to_string(),
                    alpha: Alpha { rspr: dec!(-70) },
                    terminal_packets: Some(types::Packets {
                        data: vec![
                            Some(Packet {
                                data: "1".as_bytes().to_vec(),
                            }),
                            None,
                            Some(Packet {
                                data: "3".as_bytes().to_vec(),
                            }),
                            None,
                        ],
                    }),
                    position: Pos2D {
                        x: dec!(0),
                        y: dec!(0),
                    },
                },
                types::Terminal {
                    address: "0x2".to_string(),
                    alpha: Alpha { rspr: dec!(-80) },
                    terminal_packets: Some(types::Packets {
                        data: vec![
                            None,
                            Some(Packet {
                                data: "2".as_bytes().to_vec(),
                            }),
                            None,
                            Some(Packet {
                                data: "4".as_bytes().to_vec(),
                            }),
                        ],
                    }),
                    position: Pos2D {
                        x: dec!(-1),
                        y: dec!(0),
                    },
                },
                types::Terminal {
                    address: "0x3".to_string(),
                    alpha: Alpha { rspr: dec!(-40) },
                    terminal_packets: Some(types::Packets {
                        data: vec![
                            Some(Packet {
                                data: "1".as_bytes().to_vec(),
                            }),
                            Some(Packet {
                                data: "2".as_bytes().to_vec(),
                            }),
                            Some(Packet {
                                data: "3".as_bytes().to_vec(),
                            }),
                            Some(Packet {
                                data: "4".as_bytes().to_vec(),
                            }),
                        ],
                    }),
                    position: Pos2D {
                        x: dec!(0),
                        y: dec!(2),
                    },
                },
                types::Terminal {
                    address: "0x4".to_string(),
                    alpha: Alpha { rspr: dec!(-60) },
                    terminal_packets: Some(types::Packets {
                        data: vec![None, None, None, None],
                    }),
                    position: Pos2D {
                        x: dec!(3),
                        y: dec!(0),
                    },
                },
                types::Terminal {
                    address: "0x5".to_string(),
                    alpha: Alpha { rspr: dec!(-50) },
                    terminal_packets: None,
                    position: Pos2D {
                        x: dec!(0),
                        y: dec!(-4),
                    },
                },
            ],
            satellite_packets: Some(CompletePackets {
                data: vec![
                    Packet {
                        data: "1".as_bytes().to_vec(),
                    },
                    Packet {
                        data: "2".as_bytes().to_vec(),
                    },
                    Packet {
                        data: "3".as_bytes().to_vec(),
                    },
                    Packet {
                        data: "4".as_bytes().to_vec(),
                    },
                ],
            }),
            epoch: 1,
            address: "0x123456".to_string(),
            position: types::Pos3D {
                x: dec!(0),
                y: dec!(0),
                height: dec!(10000),
            },
        };
        let zk = TestZK {};

        let satellite = Satellite::from_with_config(satellite, &cfg).unwrap();
        let pox = PoX::new(satellite, zk, &cfg).unwrap();
        let r = pox.eval_pof();
        // debug!("{:#?}", r);
        let vr = r.verify();
        assert_eq!(vr[0], PoFVerify::Success);
        assert_eq!(vr[1], PoFVerify::Success);
        assert_eq!(vr[2], PoFVerify::Success);
        assert_eq!(vr[3], PoFVerify::Success);
        assert_eq!(
            vr[4],
            PoFVerify::Fail("PoF: Terminal 0x5 Empty proof".to_string())
        );
        assert_eq!(r.value, BigInt::from(8));
        assert_eq!(
            r,
            PoFSatelliteResult::decompress_deserialize(
                &r.serialize_compress::<BrotliCompressor>(&CompressorConfig::default())
                    .unwrap(),
                &CompressorConfig::default()
            )
            .unwrap()
        );
    }
}
