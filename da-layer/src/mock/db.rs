use std::collections::HashMap;

use crate::error::Error;
use crate::{ip_packets, satellite, terminal};
use config::config::MySQLConfig;
use sea_orm::{ColumnTrait, EntityTrait, LoaderTrait, QueryFilter, QueryOrder};
use sea_orm::{JoinType, QuerySelect};
use tracing::{info, warn};

use super::prelude::*;
use super::terminal_track;

pub struct Db {
    config: MySQLConfig,
    db: sea_orm::DatabaseConnection,
}
impl Db {
    pub async fn new(config: MySQLConfig) -> Result<Self, Error> {
        info!(
            "Connecting to MySQL database at mysql://{user}@{host}:{port}/{db}",
            user = config.user,
            host = config.host,
            port = config.port,
            db = config.database
        );
        let db = sea_orm::Database::connect(config.mysql_url())
            .await
            .map_err(|e| {
                Error::DbErr(
                    format!("Connect to database {} failed", config.mysql_url()),
                    e,
                )
            })?;

        Ok(Self { config, db })
    }
}

impl Db {
    pub async fn find_all_terminal_track_with_single_satellite_block_from_to(
        &self,
        satellite_address: &str,
        block_height_from: u64,
        block_height_to: u64,
    ) -> Result<Vec<(terminal_track::Model, terminal::Model)>, Error> {
        let terminals = TerminalTrack::find()
            .filter(terminal_track::Column::BlockNumber.lte(block_height_to))
            .filter(terminal_track::Column::BlockNumber.gte(block_height_from))
            .filter(terminal_track::Column::SatelliteValidatorAddress.eq(satellite_address))
            // .inner_join(terminal::Entity)
            .find_also_related(Terminal)
            // .join(
            //     JoinType::InnerJoin,
            //     terminal_track::Entity::belongs_to(terminal::Entity)
            //         .from(terminal_track::Column::TerminalAddress)
            //         .to(terminal::Column::Address)
            //         .into(),
            // )
            .order_by_asc(terminal_track::Column::BlockNumber)
            .all(&self.db)
            .await
            .map_err(|e| {
                Error::DbErr(
                    "find_all_terminal_track_with_single_satellite_block_from_to error".to_string(),
                    e,
                )
            })?
            .into_iter()
            .filter_map(|(tt, t)| match t {
                Some(t) => Some((tt, t)),
                None => {
                    warn!(
                        "address {} in terminal_track not found in table track",
                        tt.terminal_address.as_ref().unwrap()
                    );
                    None
                }
            })
            .collect();

        Ok(terminals)
    }
    pub async fn find_all_ip_packets_with_single_satellite_block_from_to(
        &self,
        satellite_mac: &str,
        block_height_from: u64,
        block_height_to: u64,
    ) -> Result<HashMap<(String, u64), Vec<ip_packets::Model>>, Error> {
        let ip_packets = IpPackets::find()
            .filter(ip_packets::Column::BlockNumber.lte(block_height_to))
            .filter(ip_packets::Column::BlockNumber.gte(block_height_from))
            // TODO: use satellite_address
            .filter(ip_packets::Column::SatelliteMac.eq(satellite_mac))
            // .inner_join(terminal::Entity)
            // .join(
            //     JoinType::InnerJoin,
            //     terminal_track::Entity::belongs_to(terminal::Entity)
            //         .from(terminal_track::Column::TerminalAddress)
            //         .to(terminal::Column::Address)
            //         .into(),
            // )
            .order_by_asc(ip_packets::Column::BlockNumber)
            .all(&self.db)
            .await
            .map_err(|e| {
                Error::DbErr(
                    "find_all_ip_packets_with_single_satellite_block_from_to error".to_string(),
                    e,
                )
            })?
            .iter()
            .fold(HashMap::new(), |mut acc, a| {
                let key = (a.satellite_mac.to_owned(), a.block_number as u64);
                acc.entry(key).or_insert_with(Vec::new).push(a.to_owned());
                acc
            });

        Ok(ip_packets)
    }
}

#[cfg(test)]
mod tests {
    use config::config::LogConfig;
    use logger::init_logger_for_test;

    use super::*;

    #[tokio::test]
    async fn test_db() {
        let _guard = init_logger_for_test!();
        let cfg = config::config::Config::new().unwrap();
        let _ = Db::new({
            let config::config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg.da_layer;
            cfg
        })
        .await
        .unwrap();
    }
    #[tokio::test]
    // #[cfg(exclude)]
    async fn test_db_find_all_terminal_track_with_single_satellite_block_from_to() {
        let _guard = init_logger_for_test!();

        let cfg = config::config::Config::new().unwrap();
        let db = Db::new({
            let config::config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg.da_layer;
            cfg
        })
        .await
        .unwrap();
        let result = db
            .find_all_terminal_track_with_single_satellite_block_from_to(
                "evmosvaloper1q9dvfsksdv88yz8yjzm6xy808888ylc8e2n838",
                180000,
                500715,
            )
            .await
            .unwrap();
        info!("result len: {}", result.len());
    }
    #[tokio::test]
    // #[cfg(exclude)]
    async fn test_db_find_all_ip_packets_with_single_satellite_block_from_to() {
        let _guard = init_logger_for_test!();

        let cfg = config::config::Config::new().unwrap();
        let db = Db::new({
            let config::config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg.da_layer;
            cfg
        })
        .await
        .unwrap();
        let result = db
            .find_all_ip_packets_with_single_satellite_block_from_to(
                "6C:AC:B2:55:09:A5",
                180000,
                500715,
            )
            .await
            .unwrap();
        info!("result len: {}", result.len());
    }
}
