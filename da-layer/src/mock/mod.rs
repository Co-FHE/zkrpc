mod db;
mod models;

pub use models::*;
use rust_decimal::Decimal;
use tracing::{error, warn};
use types::{CompletePackets, Packet, Packets, Terminal};

use crate::{DaLayerTrait, Error};
use proj::{Coord, Proj};

struct MockLocalDB {
    db: db::Db,
    proj: Proj,
}
impl DaLayerTrait for MockLocalDB {
    async fn fetch_satellite_with_terminals_block_from_to(
        &self,
        satellite_address: &str,
        block_height_from: u64,
        block_height_to: u64,
    ) -> Result<types::Satellite<Decimal>, crate::Error> {
        let terminals = self
            .db
            .find_all_terminal_track_with_single_satellite_block_from_to(
                satellite_address,
                block_height_from,
                block_height_to,
            )
            .await?;
        let (terminals, (height_mac_index, dropped_packets_indices)): (Vec<_>, (Vec<_>, Vec<_>)) =
            terminals
                .into_iter()
                .filter_map(|(x, y)| {
                    x.terminal_address.and_then(|ta| {
                        Some((
                            {
                                let pos = self
                                    .proj
                                    .convert((y.longitude as f64, y.latitude as f64))
                                    .map_err(|e| {
                                        let err = crate::Error::LatLonErr(
                                            y.latitude as f64,
                                            y.longitude as f64,
                                            e,
                                        );
                                        error!("{}", err);
                                        err
                                    })
                                    .ok()?;

                                Terminal::<Decimal>::new_from_f64(
                                    ta,
                                    pos.x() as f64,
                                    pos.y() as f64,
                                    x.signal_strength as f64,
                                )
                                .ok()?
                            },
                            (
                                (x.satellite_mac, x.block_number as u64),
                                x.droped_ip_packets,
                            ),
                        ))
                    })
                })
                .unzip();
        let satellite_packets = self
            .db
            .find_all_ip_packets_with_single_satellite_block_from_to(
                satellite_address,
                block_height_from,
                block_height_to,
            )
            .await?;
        let satellite_packets: Vec<_> = height_mac_index
            .iter()
            .map(|(mac, height)| {
                satellite_packets
                    .get(&(mac.to_owned(), *height))
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
                    })
            })
            .collect();
        let dropped_packets_indices: Vec<_> = dropped_packets_indices
            .into_iter()
            .map(|indices| {
                indices.and_then(|indices| {
                    if indices.is_empty() {
                        return Some(vec![]);
                    }
                    let r = indices
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
                        .ok()?;
                    Some(r)
                })
            })
            .collect();
        assert!(dropped_packets_indices.len() == satellite_packets.len());
        let terminal_packets = dropped_packets_indices
            .into_iter()
            .zip(satellite_packets.iter())
            .map(|(indices, packets)| {
                packets.clone().and_then(|packets| {
                    indices.and_then(|indices| {
                        let mut p = packets.data.into_iter().map(Some).collect::<Vec<_>>();
                        indices.iter().for_each(|i| {
                            p[*i] = None;
                        });
                        Some(Packets { data: p })
                    })
                })
            })
            .collect::<Vec<_>>();
        Ok(types::Satellite::<Decimal> {
            address: satellite_address.to_string(),
            terminals,
            satellite_packets,
            terminal_packets,
        })
    }

    async fn new(cfg: &config::config::DaLayerConfig) -> Result<Self, crate::Error> {
        let from = "EPSG:4326";
        let to = "EPSG:3309";
        let proj = Proj::new_known_crs(&from, &to, None).map_err(|e| {
            let err = crate::Error::ProjErr(from.to_string(), to.to_string(), e);
            error!("{}", err);
            err
        })?;
        if let config::config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg {
            let db = db::Db::new(cfg.clone()).await?;
            Ok(Self { db, proj })
        } else {
            Err(crate::Error::ConfigErr(
                "the DaLayerConfig is not MockDaLayerConfig".to_string(),
            ))
        }
    }
}
