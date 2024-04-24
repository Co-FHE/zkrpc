use crate::mock::error::Error;
use config::config::MySQLConfig;
use tracing::info;

struct Db {
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

#[cfg(test)]
mod tests {
    use config::config::LogConfig;

    use super::*;

    #[tokio::test]
    async fn test_db() {
        let _guard = logger::initialize_logger(&LogConfig::default());
        let cfg = config::config::Config::new().unwrap();
        let _ = Db::new({
            let config::config::DaLayerConfig::MockDaLayerConfig(cfg) = cfg.da_layer;
            cfg
        })
        .await
        .unwrap();
    }
}
