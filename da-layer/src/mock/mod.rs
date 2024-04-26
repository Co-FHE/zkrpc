mod db;
mod models;
pub use models::*;
use rust_decimal::Decimal;
use tracing::error;
use types::Terminal;

use crate::DaLayerTrait;
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
        let result = self
            .db
            .find_all_terminal_track_with_single_satellite_block_from_to(
                satellite_address,
                block_height_from,
                block_height_to,
            )
            .await?;
        let t = result
            .into_iter()
            .filter_map(|(x, y)| {
                x.terminal_address.and_then(|ta| {
                    Some((ta.clone(), {
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
                    }))
                })
            })
            .collect();
        Ok(types::Satellite::<Decimal> {
            address: satellite_address.to_string(),
            terminals: t,
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
