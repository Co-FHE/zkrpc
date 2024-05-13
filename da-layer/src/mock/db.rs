use crate::error::Error;
use crate::{ip_packets, p2p, satellite_track};
use config::MySQLConfig;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, EntityTrait, QueryFilter, QueryOrder, Set,
};
use tracing::*;

use super::prelude::*;
use super::terminal_track;

#[derive(Clone, Debug)]
pub struct Db {
    db: sea_orm::DatabaseConnection,
}
impl Db {
    pub async fn new(config: &MySQLConfig) -> Result<Self, Error> {
        // let _span = debug_span!("init_mysql").entered();
        debug!("Connecting to MySQL...");
        let mut db_opt = ConnectOptions::new(config.mysql_url());
        if let Some(log_level) = &config.sqlx_log_level_filter {
            db_opt.sqlx_logging_level(log_level.clone().into());
        } else {
            db_opt.sqlx_logging(false);
        }

        let db = sea_orm::Database::connect(db_opt).await.map_err(|e| {
            Error::DbErr(
                format!("Connect to database {} failed", config.mysql_url()),
                e,
            )
        })?;

        debug!(
            message = %"MySQL Connected",
            db = format!(
                "mysql://{user}@{host}:{port}/{db}",
                user = config.user,
                host = config.host,
                port = config.port,
                db = config.database
            ),
            sqlx_level = config
                .clone()
                .sqlx_log_level_filter
                .map_or("disabled".to_string(), |x| x.to_string()),
        );

        Ok(Self { db })
    }
}

impl Db {
    pub async fn upsert_p2p(
        &self,
        address: String,
        multi_addr: String,
        peer_id: String,
    ) -> Result<(), Error> {
        let record = p2p::ActiveModel {
            address: Set(address.clone()),
            peer_id: Set(peer_id),
            multi_addr: Set(multi_addr),
            ..Default::default()
        };

        match p2p::Entity::find_by_id(address.clone()).one(&self.db).await {
            Ok(Some(_)) => match record.update(&self.db).await {
                Ok(_) => Ok(()),
                Err(err) => Err(Error::DbErr("update p2p error".to_string(), err)),
            },
            _ => match record.insert(&self.db).await {
                Ok(_) => Ok(()),
                Err(err) => Err(Error::DbErr("insert p2p error".to_string(), err)),
            },
        }
    }
    pub async fn find_p2p_by_address(&self, address: &str) -> Result<Option<p2p::Model>, Error> {
        p2p::Entity::find_by_id(address.to_string())
            .one(&self.db)
            .await
            .map_err(|e| Error::DbErr("find_p2p_by_address error".to_string(), e))
    }
    pub async fn remove_p2p_by_address(&self, address: &str) -> Result<(), Error> {
        p2p::Entity::delete_by_id(address)
            .exec(&self.db)
            .await
            .map_err(|e| Error::DbErr("remove_p2p_by_address error".to_string(), e))?;
        Ok(())
    }
    pub async fn find_all_satellite_track_with_single_satellite_block_from_to(
        &self,
        satellite_address: &str,
        block_height_from: u64,
        block_height_to: u64,
    ) -> Result<Vec<satellite_track::Model>, Error> {
        satellite_track::Entity::find()
            .filter(satellite_track::Column::BlockNumber.lte(block_height_to))
            .filter(satellite_track::Column::BlockNumber.gte(block_height_from))
            .filter(satellite_track::Column::ValidatorAddress.eq(satellite_address))
            .order_by_asc(satellite_track::Column::BlockNumber)
            .all(&self.db)
            .await
            .map_err(|e| {
                Error::DbErr(
                    "find_all_satellite_track_with_single_statellite_block_from_to error"
                        .to_string(),
                    e,
                )
            })
    }
    pub async fn find_all_terminal_track_with_single_satellite_block_from_to(
        &self,
        satellite_address: &str,
        block_height_from: u64,
        block_height_to: u64,
    ) -> Result<Vec<terminal_track::Model>, Error> {
        let terminals = TerminalTrack::find()
            .filter(terminal_track::Column::BlockNumber.lte(block_height_to))
            .filter(terminal_track::Column::BlockNumber.gte(block_height_from))
            .filter(terminal_track::Column::SatelliteValidatorAddress.eq(satellite_address))
            // .inner_join(terminal::Entity)
            // .find_also_related(Terminal)
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
            .collect();

        Ok(terminals)
    }
    pub async fn find_all_ip_packets_with_single_satellite_block_from_to(
        &self,
        satellite_address: &str,
        block_height_from: u64,
        block_height_to: u64,
    ) -> Result<Vec<ip_packets::Model>, Error> {
        let ip_packets = IpPackets::find()
            .filter(ip_packets::Column::BlockNumber.lte(block_height_to))
            .filter(ip_packets::Column::BlockNumber.gte(block_height_from))
            // TODO: use satellite_address
            .filter(ip_packets::Column::SatelliteValidatorAddress.eq(satellite_address))
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
            })?;

        Ok(ip_packets)
    }
}

#[cfg(test)]
mod tests {
    use logger::init_logger_for_test;

    use super::*;

    #[tokio::test]
    async fn test_db() {
        let _guard = init_logger_for_test!();
        let cfg = config::Config::new().unwrap();
        if let config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg.da_layer {
            let _ = Db::new(&cfg).await.unwrap();
        } else {
            panic!("cfg.da_layer should be MockDaLayerConfig");
        }
    }
    #[tokio::test]
    // #[cfg(exclude)]
    async fn test_db_find_all_satellite_track_with_single_satellite_block_from_to() {
        let _guard = init_logger_for_test!();

        let cfg = config::Config::new().unwrap();
        if let config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg.da_layer {
            let db = Db::new(&cfg).await.unwrap();
            let result = db
                .find_all_satellite_track_with_single_satellite_block_from_to(
                    "space1fdhkvj4zjgverz2fsy6cmehxx6gtxrwh0j7pch",
                    0,
                    500715,
                )
                .await
                .unwrap();
            info!("result len: {}", result.len());
        } else {
            panic!("cfg.da_layer should be MockDaLayerConfig");
        }
    }
    #[tokio::test]
    // #[cfg(exclude)]
    async fn test_db_find_all_terminal_track_with_single_satellite_block_from_to() {
        let _guard = init_logger_for_test!();

        let cfg = config::Config::new().unwrap();
        if let config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg.da_layer {
            let db = Db::new(&cfg).await.unwrap();
            let result = db
                .find_all_terminal_track_with_single_satellite_block_from_to(
                    "space1fdhkvj4zjgverz2fsy6cmehxx6gtxrwh0j7pch",
                    0,
                    500715,
                )
                .await
                .unwrap();
            info!("result len: {}", result.len());
        } else {
            panic!("cfg.da_layer should be MockDaLayerConfig");
        }
    }
    #[tokio::test]
    // #[cfg(exclude)]
    async fn test_db_find_all_ip_packets_with_single_satellite_block_from_to() {
        let _guard = init_logger_for_test!();

        let cfg = config::Config::new().unwrap();
        if let config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg.da_layer {
            let db = Db::new(&cfg).await.unwrap();
            let result = db
                .find_all_ip_packets_with_single_satellite_block_from_to(
                    "6C:AC:B2:55:09:A5",
                    180000,
                    500715,
                )
                .await
                .unwrap();
            info!("result len: {}", result.len());
        } else {
            panic!("cfg.da_layer should be MockDaLayerConfig");
        }
    }
    #[tokio::test]
    async fn test_db_p2p() {
        let _guard = init_logger_for_test!();
        let cfg = config::Config::new().unwrap();
        if let config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg.da_layer {
            let db = Db::new(&cfg).await.unwrap();
            let address = "test_address".to_string();
            let multi_addr = "test_multi_addr".to_string();
            let peer_id = "test_peer_id".to_string();
            db.upsert_p2p(address.clone(), multi_addr.clone(), peer_id.clone())
                .await
                .unwrap();
            let p2p = db.find_p2p_by_address(&address).await.unwrap().unwrap();
            assert_eq!(p2p.address, address);
            assert_eq!(p2p.multi_addr, multi_addr);
            assert_eq!(p2p.peer_id, peer_id);
            let multi_addr = "test_multi_addr2".to_string();
            let peer_id = "test_peer_id2".to_string();
            db.upsert_p2p(address.clone(), multi_addr.clone(), peer_id.clone())
                .await
                .unwrap();
            let p2p = db.find_p2p_by_address(&address).await.unwrap().unwrap();
            assert_eq!(p2p.address, address);
            assert_eq!(p2p.multi_addr, multi_addr);
            assert_eq!(p2p.peer_id, peer_id);
            db.remove_p2p_by_address(&address).await.unwrap();
            let p2p = db.find_p2p_by_address(&address).await.unwrap();
            assert_eq!(p2p, None);
        } else {
            panic!("cfg.da_layer should be MockDaLayerConfig");
        }
    }
}
