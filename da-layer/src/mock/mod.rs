mod db;
mod models;

use std::{collections::HashMap, f32::consts::E};

use flat_projection::*;
pub use models::*;
use rust_decimal::Decimal;
use tracing::{error, warn};
use types::{CompletePackets, Packet, Packets, Pos2D, Pos3D, Satellite, Terminal};

use crate::{DaLayerTrait, Error};
// use proj::{Coord, Proj};

#[derive(Debug,Clone)]
pub struct MockLocalDB {
    db: db::Db,
}
impl DaLayerTrait for MockLocalDB {
    async fn fetch_satellite_with_terminals_block_from_to(
        &self,
        satellite_address: &str,
        block_height_from: u64,
        block_height_to: u64,
    ) -> Result<Vec<(usize,types::Satellite<Decimal>)>, crate::Error> {
        // TODO: address may have lower case or upper case problem
        let satellites = self
            .db
            .find_all_satellite_track_with_single_satellite_block_from_to(
                satellite_address,
                block_height_from,
                block_height_to,
            )
            .await?
            .into_iter()
            .fold(HashMap::new(), |mut acc, satellite| {
                let key = (
                    satellite.block_number as u64,
                    satellite.validator_address.clone(),
                );
                if !acc.contains_key(&key) {
                    acc.insert(key, satellite);
                } else {
                    warn!(
                        "Warning: Duplicate entry with key {:?}, has {:?}, drop {:?}",
                        key,
                        acc.get(&key),
                        satellite
                    );
                }
                acc
            });
        let terminals = self
            .db
            .find_all_terminal_track_with_single_satellite_block_from_to(
                satellite_address,
                block_height_from,
                block_height_to,
            )
            .await?
            .into_iter()
            .fold(HashMap::new(), |mut acc, (terminal_track, terminal)| {
                assert!(terminal.address == terminal_track.terminal_address);
                acc.entry((
                    terminal_track.block_number as u64,
                    terminal_track.satellite_validator_address.to_owned(),
                ))
                .or_insert_with(Vec::new)
                .push((terminal_track, terminal));
                acc
            });
        let satellite_packets = self
            .db
            .find_all_ip_packets_with_single_satellite_block_from_to(
                satellite_address,
                block_height_from,
                block_height_to,
            )
            .await?
            .into_iter()
            .fold(HashMap::new(), |mut acc, ip_packet| {
                acc.entry((
                    ip_packet.block_number as u64,
                    ip_packet.satellite_validator_address.clone(),
                ))
                .or_insert_with(Vec::new)
                .push(ip_packet);
                acc
            });
        let mut satellites:Vec<_> = satellites
            .into_iter()
            .filter_map(|(blocknum_saddress, satellite_track)| {
                let satellite_position = Pos3D::<Decimal>::new_from_f64(
                    satellite_track.longitude as f64,
                    satellite_track.latitude as f64,
                    satellite_track.height as f64,
                ).map_err(|e| {
                    let err = crate::Error::TypesError(e);
                    error!("{}", err);
                    err
                }).ok()?;
                let proj = FlatProjection::<f64>::new(
                    satellite_track.longitude as f64,
                    satellite_track.latitude as f64,
                );
                let satellite_packets =
                    satellite_packets
                        .get(&blocknum_saddress)
                        .and_then(|packet_model| {
                            let mut packet_model = packet_model
                                .iter()
                                .map(|packet_model| {
                                    (
                                        packet_model.ip_sequence as usize,
                                        Packet {
                                            data: packet_model.ip_packet_data.clone(),
                                        },
                                    )
                                })
                                .collect::<Vec<_>>();
                            packet_model.sort_by_key(|(ip_sequence, _)| *ip_sequence);
                            let (ip_sequence, data): (Vec<_>, Vec<_>) =
                                packet_model.into_iter().unzip();
                            //check ip_sequence is continuous
                            if ip_sequence.iter().enumerate().all(|(i, v)| i == *v) {
                                Some(CompletePackets { data })
                            } else {
                                warn!(
                                    "ip_sequence is not continuous for satellite {}",
                                    satellite_address
                                );
                                None
                            }
                            //     .max_by_key(|m| m.ip_sequence)
                            //     .and_then(|max_seq| {
                            //         packet_model.iter().all(|packet_model| {
                            //             (packet_model.ip_sequence as usize, packet_model.ip_sequence)
                            //                 == (max_seq.ip_sequence as usize, max_seq.ip_sequence
                            //         });
                            //         if max_seq.ip_sequence as usize == packet_model.len() {
                            //             let mut r = vec![None; packet_model.len()];
                            //             //TODO: need to check the sequence is continuous for satellite
                            //             packet_model.iter().filter_map(|packet_model| {
                            //                 r[packet_model.ip_sequence as usize] = Packet::Packet {
                            //                     data: packet_model.ip_packet_data.clone(),
                            //                 }
                            //             });
                            //             Some(Packets { data: r })
                            //         } else {
                            //             None
                            //         }
                            //     })
                        });
                let terminals = terminals
                    .get(&blocknum_saddress)
                    .map_or(Vec::new(), |terminals| {
                        let terminals = terminals
                            .iter()
                            .filter_map(|(terminal_track, terminal)| {
                                assert!(
                                    terminal_track.block_number as u64 == blocknum_saddress.0
                                        && terminal_track.satellite_validator_address
                                            == blocknum_saddress.1
                                        && terminal.address == terminal_track.terminal_address
                                );
                                let pos = proj
                                    .project(terminal.longitude as f64, terminal.latitude as f64);
                                // let pos = Pos2D::<Decimal>::new_from_flat_point_f64(pos)
                                //     .map_err(|e| {
                                //         let err = crate::Error::TypesError(e);
                                //         error!("{}", err);
                                //         err
                                //     })
                                //     .ok()?;
                                let terminal_packets = if let Some(satellite_packets) = &satellite_packets {
                                    terminal_track.droped_ip_packets.as_ref().and_then(|indices| {
                                        let dropped_indices =
                                            if indices.is_empty() { vec![] } else { 
                                                indices
                                                .split(",")
                                                .map(|s| {
                                                    s.parse::<usize>().map_err(|e| {
                                                        warn!(
                                                            "parse dropped index {} error: {} for satellite {}",
                                                            s, e, satellite_address
                                                        );
                                                        Error::ParseErr(s.to_string(), e)
                                                    })
                                                })
                                                .collect::<Result<Vec<_>, _>>()
                                                .ok()?
                                            };
                                        let mut p = satellite_packets
                                            .clone()
                                            .data
                                            .into_iter()
                                            .map(Some)
                                            .collect::<Vec<_>>();
                                        dropped_indices.iter().for_each(|i| {
                                            p[*i] = None;
                                        });
                                        Some(Packets { data: p })
                                    })
                                }else{
                                    None
                                };

                                Terminal::<Decimal>::new_from_f64(
                                    terminal_track.terminal_address.clone(),
                                    pos.x as f64,
                                    pos.y as f64,
                                    terminal_track.signal_strength as f64,
                                    terminal_packets,
                                )
                                .map_err(|e| {
                                    let err = crate::Error::TypesError(e);
                                    error!("{}", err);
                                    err
                                })
                                .ok()
                            })
                            .collect();
                        terminals
                    });
                Some(Satellite::<Decimal> {
                    epoch: blocknum_saddress.0 as usize,
                    address: satellite_track.validator_address.clone(),
                    position: satellite_position,
                    terminals,
                    satellite_packets,
                })
            })
            .fold(HashMap::new(), |mut acc, satellite|{
                if acc.contains_key(&satellite.epoch){
                    warn!("Warning: Duplicate entry with key {:?}, has {:?}, drop {:?}", satellite.epoch, acc.get(&satellite.epoch), satellite);
                }else{
                    acc.insert(satellite.epoch, satellite);

                }
                acc
            }).into_iter().collect();

            // 根据usize排序
            satellites.sort_by_key(|pair| pair.0); 
        Ok(satellites)
    }

    async fn new(cfg: &config::config::DaLayerConfig) -> Result<Self, crate::Error> {
        // let from = "EPSG:4326";
        // let to = "EPSG:3309";
        // let proj = Proj::new_known_crs(&from, &to, None).map_err(|e| {
        //     let err = crate::Error::ProjErr(from.to_string(), to.to_string(), e);
        //     error!("{}", err);
        //     err
        // })?;
        if let config::config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg {
            let db = db::Db::new(&cfg).await?;
            Ok(Self { db })
        } else {
            Err(crate::Error::ConfigErr(
                "the DaLayerConfig is not MockDaLayerConfig".to_string(),
            ))
        }
    }
}
#[cfg(test)]
mod tests {
    use config::config::LogConfig;
    use logger::init_logger_for_test;
    use tracing::{debug, info};

    use super::*;

    #[tokio::test]
    async fn test_db() {
        let _guard = init_logger_for_test!();
        let cfg = config::config::Config::new().unwrap();
        let _ = MockLocalDB::new(&cfg.da_layer)
            .await
            .expect("create MockLocalDB failed");
    }
    #[tokio::test]
    async fn test_fetch_satellite_with_terminals_block_from_to() {
        let _guard = init_logger_for_test!();
        let cfg = config::config::Config::new().unwrap();
        let db = MockLocalDB::new(&cfg.da_layer)
            .await
            .expect("create MockLocalDB failed");
        let results = db
            .fetch_satellite_with_terminals_block_from_to(
                "evmosvaloper1q9dvfsksdv88yz8yjzm6xy808888ylc8e2n838",
                180000,
                500715,
            )
            .await
            .unwrap();
        results.iter().for_each(|(height,result)| debug!(height=height, 
        satellite_address=result.address, 
        terminals_num=result.terminals.len(),
        satellite_packets=
        match &result.satellite_packets {
            Some(p) => p.data.len(),
            None => 0,
        },
        valid_terminal_packets=?result.terminals.iter().map(|t| {
            match &t.terminal_packets {
                Some(p) => p.data.iter().filter(|p| p.is_some()).count(),
                None => 0,
            }
        }).collect::<Vec<_>>(),
        "DA Data"));
    }
    // #[test]
    // fn test_proj() {
    //     let from = "EPSG:4326";
    //     let to = "EPSG:3309";
    //     let proj = Proj::new_known_crs(&from, &to, None).unwrap();
    //     let pos = proj.convert((8.22, 47.33)).unwrap();
    //     println!("{:?}", pos);
    // }
}
