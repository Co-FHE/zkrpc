use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error db {0}: {1}")]
    DbErr(String, #[source] sea_orm::DbErr),
}
