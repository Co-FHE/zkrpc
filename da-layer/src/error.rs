use std::num::ParseIntError;

use proj::{ProjCreateError, ProjError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error db {0}: {1}")]
    DbErr(String, #[source] sea_orm::DbErr),
    #[error("Config error: {0}")]
    ConfigErr(String),
    #[error("lat-lon error: lat-{0}, lon-{1}, {2}")]
    LatLonErr(f64, f64, #[source] ProjError),
    #[error("proj convert error: {0}, {1}, {2}")]
    ProjErr(String, String, #[source] ProjCreateError),
    #[error("Parse error: {0}, {1}")]
    ParseErr(String, #[source] ParseIntError),
}
