use serde::{Deserialize, Serialize};

use crate::LogLevel;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub enum DaLayerConfig {
    MockDaLayerConfig(MySQLConfig),
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct MySQLConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub sqlx_log_level_filter: Option<LogLevel>,
}
impl Default for MySQLConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_owned(),
            port: 3306,
            user: "root".to_owned(),
            password: "root".to_owned(),
            database: "test".to_owned(),
            sqlx_log_level_filter: LogLevel::Debug.into(),
        }
    }
}
impl Default for DaLayerConfig {
    fn default() -> Self {
        DaLayerConfig::MockDaLayerConfig(MySQLConfig::default())
    }
}
impl MySQLConfig {
    pub fn mysql_url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}
