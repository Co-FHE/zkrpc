#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use config::*;
    use halo2_proofs::pasta::Fp;
    use hdrhistogram::Histogram;
    use logger::init_logger_for_test;
    use num_bigint::BigInt;
    use num_rational::BigRational;
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use rust_decimal::{prelude::Zero, Decimal};
    use rust_decimal_macros::dec;
    use tracing::{debug, info};
    use types::{Alpha, CompletePackets, EndPointFrom, Packet, Pos2D, Satellite};
    use util::{compressor::BrotliCompressor, serde_bin::SerdeBinTrait};
    use zkt::ZkTraitHalo2;

    use crate::{
        Gaussian, GaussianTaylor, Kernel, KernelKind, PoDSatelliteResult, PoDTerminalResult,
        PoFSatelliteResult, PoFVerify, PoX, PosTrait,
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
    /*
           Y
           |                   *
       2   |                   ( 0, 2) -40
           |
       1   |
           |
    -------|-------*-----------*----------------------*------ X
           |   (-1, 0) -80   ( 0, 0) -70           ( 3, 0) -60
      -1   |
           |
      -2   |
           |
      -3   |
           |
      -4   |                   *
           |                  ( 0,-4) -50
           |
    */
    /*
       dis_sqr(0,1) = 1
       dis_sqr(0,2) = 4
       dis_sqr(0,3) = 9
       dis_sqr(0,4) = 16
       dis_sqr(1,2) = 5
       dis_sqr(1,3) = 16
       dis_sqr(1,4) = 17 (x)
       dis_sqr(2,3) = 13 (x)
       dis_sqr(2,4) = 36 (x)
       dis_sqr(3,4) = 25 (x)
    */
    /*
    sigma_sqr = 4
    order=1
    sigma_sqr_range = 2^2*2^2 = 16
    taylor: (2s^2-x^2)/2s^2
    taylor(0,1) = (8-1)/8 = 7/8
    taylor(0,2) = (8-4)/8 = 1/2
    taylor(0,3) = (8-9)/8 = -1/8 = 0
    taylor(0,4) = (8-16)/8 = -1 = 0
    taylor(1,2) = (8-5)/8 = 3/8
    taylor(1,3) = (8-16)/8 = -1 = 0
    taylor(1,4) = (8-17)/8 = -9/8 = 0
    taylor(2,3) = (8-13)/8 = -5/8 = 0
    taylor(2,4) = (8-36)/8 = -4 = 0
    taylor(3,4) = (8-25)/8 = -17/8 = 0
    */
    /*
       0: (8*(-70)+7*(-80)+4*(-40))/(8+7+4)= -560-560-160/19 = -1280/19=67.3684
       1: (7*(-70)+8*(-80)+3*(-40))/(7+8+3)= -490-640-120/18 = -1250/18 = 69.4444
       2: (4*(-70)+3*(-80)+8*(-40))/(4+3+8)= -280-240-320/15 = -840/15 = 56
    */
    #[test]
    fn test_pod() {
        let _guard = init_logger_for_test!();
        use crate::PoX;
        use config::PoxConfig;
        use types::{Alpha, FixedPoint, Satellite};
        let mut cfg = PoxConfig {
            kernel: KernelConfig {
                quadratic: QuadraticConfig {
                    max_dis_sqr: dec!(25),
                },
                gaussian: GaussianConfig {
                    sigma: dec!(2),
                    vanilla: GaussianVanillaConfig { use_coef: false },
                    taylor: GaussianTaylorConfig {
                        max_order: 1,
                        sigma_range: dec!(2.0),
                    },
                },
                kernel_type: KernelTypeConfig::Quadratic,
            },
            penalty: PenaltyConfig { max_diff: dec!(20) },
            rspr_precision_bigint: 4,
            coordinate_precision_bigint: 3,
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
            score: BigInt::from(384606),
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
        let pox = PoX::new(satellite.clone(), TestZK {}, &cfg).unwrap();
        if let KernelKind::Quadratic(kernel) = &pox.kernel {
            assert_eq!(kernel.max_dis_sqr, BigInt::from(25_000_000));
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
        } else {
            panic!("KernelKind is not Quadratic")
        }
        cfg.kernel.kernel_type = KernelTypeConfig::GaussianTaylor;
        let pox = PoX::new(satellite.clone(), TestZK {}, &cfg).unwrap();
        if let KernelKind::GaussianTaylor(kernel) = &pox.kernel {
            assert_eq!(kernel.sigma_sqr, BigInt::from(4000000));
            assert_eq!(kernel.implement_params.max_order, 1);
            assert_eq!(
                kernel.implement_params.sigma_range,
                BigRational::from_integer(BigInt::from(2)),
            );
        } else {
            panic!("KernelKind is not Gaussian")
        }
        let kernel = Gaussian::<BigInt, GaussianTaylor>::from_pox_cfg(&cfg).unwrap();
        info!("{}", kernel.denom());
        let mut hm: HashMap<(usize, usize), BigInt> = HashMap::new();
        for i in 0..5 {
            for j in 0..5 {
                hm.insert(
                    (i, j),
                    kernel.eval_numer(
                        &satellite.terminals[i].position,
                        &satellite.terminals[j].position,
                    ),
                );
            }
        }
        assert_eq!(hm[&(0, 0)], BigInt::from(8000000));
        assert_eq!(hm[&(0, 1)], BigInt::from(7000000));
        assert_eq!(hm[&(0, 2)], BigInt::from(4000000));
        assert_eq!(hm[&(0, 3)], BigInt::zero());
        assert_eq!(hm[&(0, 4)], BigInt::zero());
        assert_eq!(hm[&(1, 0)], BigInt::from(7000000));
        assert_eq!(hm[&(1, 1)], BigInt::from(8000000));
        assert_eq!(hm[&(1, 2)], BigInt::from(3000000));
        assert_eq!(hm[&(1, 3)], BigInt::zero());
        assert_eq!(hm[&(1, 4)], BigInt::zero());
        assert_eq!(hm[&(2, 0)], BigInt::from(4000000));
        assert_eq!(hm[&(2, 1)], BigInt::from(3000000));
        assert_eq!(hm[&(2, 2)], BigInt::from(8000000));
        assert_eq!(hm[&(2, 3)], BigInt::zero());
        assert_eq!(hm[&(2, 4)], BigInt::zero());
        assert_eq!(hm[&(3, 0)], BigInt::zero());
        assert_eq!(hm[&(3, 1)], BigInt::zero());
        assert_eq!(hm[&(3, 2)], BigInt::zero());
        assert_eq!(hm[&(3, 3)], BigInt::from(8000000));
        assert_eq!(hm[&(3, 4)], BigInt::zero());
        assert_eq!(hm[&(4, 0)], BigInt::zero());
        assert_eq!(hm[&(4, 1)], BigInt::zero());
        assert_eq!(hm[&(4, 2)], BigInt::zero());
        assert_eq!(hm[&(4, 3)], BigInt::zero());
        assert_eq!(hm[&(4, 4)], BigInt::from(8000000));
        let pod_result = pox.eval_pod();

        let required_result = PoDSatelliteResult::<BigInt> {
            score: BigInt::from(399834),
            terminal_results: vec![
                PoDTerminalResult {
                    terminal_address: "0x1".to_string(),
                    weight: BigInt::from(173685),
                    value_for_satellite: BigInt::from(-673684),
                    proof: (Vec::new(), Vec::new()),
                },
                PoDTerminalResult {
                    terminal_address: "0x2".to_string(),
                    weight: BigInt::from(94445),
                    value_for_satellite: BigInt::from(-694444),
                    proof: (Vec::new(), Vec::new()),
                },
                PoDTerminalResult {
                    terminal_address: "0x3".to_string(),
                    weight: BigInt::from(40000),
                    value_for_satellite: BigInt::from(-560000),
                    proof: (Vec::new(), Vec::new()),
                },
                PoDTerminalResult {
                    terminal_address: "0x4".to_string(),
                    weight: BigInt::from(200000),
                    value_for_satellite: BigInt::from(-600000),
                    proof: (Vec::new(), Vec::new()),
                },
                PoDTerminalResult {
                    terminal_address: "0x5".to_string(),
                    weight: BigInt::from(200000),
                    value_for_satellite: BigInt::from(-500000),
                    proof: (Vec::new(), Vec::new()),
                },
            ],
        };
        assert_eq!(pod_result, required_result);
    }
    #[test]
    // #[cfg(not(debug_assertions))]
    fn test_pod_benchmark() {
        const N: usize = 1000;
        let _guard = init_logger_for_test!();
        use crate::PoX;
        use config::PoxConfig;
        use types::{Alpha, Satellite};
        let cfg = PoxConfig::default();
        let satellite = Satellite::<Decimal> {
            terminals: (0..N)
                .map(|i| {
                    types::Terminal {
                        //random string
                        address: format!("0x{}", i),
                        alpha: Alpha { rspr: dec!(-70) },
                        terminal_packets: None,
                        position: Pos2D {
                            x: dec!(0),
                            y: dec!(0),
                        },
                    }
                })
                .collect(),

            satellite_packets: None,
            epoch: 1,
            address: "0x123456".to_string(),
            position: types::Pos3D {
                x: dec!(0),
                y: dec!(0),
                height: dec!(10000),
            },
        };
        let satellite = Satellite::from_with_config(satellite, &cfg).unwrap();
        let pox = PoX::new(satellite, TestZK {}, &cfg).unwrap();
        let pod_result = pox.eval_pod();
        assert_eq!(pod_result.terminal_results.len(), N);
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
    ///
    /// cargo test --package pox --lib --release -- tests::tests::test_pof_benchmark --exact --show-output
    #[test]
    fn test_pof_benchmark() {
        let _guard = init_logger_for_test!();
        const N: usize = 10000;
        const PSIZE: usize = 5000;
        let cfg = PoxConfig::default();
        let satellite = Satellite::<Decimal> {
            terminals: (0..N)
                .collect::<Vec<usize>>()
                .par_iter()
                // .progress_count(N as u64)
                .map(|i| types::Terminal {
                    address: format!("0x{}", i),
                    alpha: Alpha { rspr: dec!(-70) },
                    terminal_packets: Some(types::Packets {
                        data: {
                            let mut a = vec![
                                Some(Packet {
                                    data: "1".as_bytes().to_vec()
                                });
                                PSIZE
                            ];

                            for i in 0..100 {
                                a[i] = None;
                            }
                            a
                        },
                    }),
                    position: Pos2D {
                        x: dec!(0),
                        y: dec!(0),
                    },
                })
                .collect(),

            satellite_packets: Some(CompletePackets {
                data: vec![
                    Packet {
                        data: "1".as_bytes().to_vec(),
                    };
                    PSIZE
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
            r,
            PoFSatelliteResult::decompress_deserialize(
                &r.serialize_compress::<BrotliCompressor>(&CompressorConfig::default())
                    .unwrap(),
                &CompressorConfig::default()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_histogram() {
        use hdrhistogram::SyncHistogram;
        use rayon::prelude::*;
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut histogram = SyncHistogram::<u64>::from(Histogram::new(3).unwrap());

        data.par_iter().for_each(|&value| {
            histogram
                .recorder()
                .record(value)
                .expect("failed to record value");
        });
        histogram.refresh();
        println!("# of samples: {}", histogram.len());
        println!("99.9'th percentile: {}", histogram.value_at_quantile(0.995));
        for v in histogram.iter_recorded() {
            println!(
                "{}'th percentile of data is {} with {} samples",
                v.percentile(),
                v.value_iterated_to(),
                v.count_at_value()
            );
        }
    }
}
