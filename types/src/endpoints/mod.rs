mod terminal;
use config::PoxConfig;
pub use terminal::*;
mod satellite;
pub use satellite::*;

mod packets;
pub use packets::*;

use crate::{Error, FixedPoint};

pub trait EndPointFrom<T>: Sized {
    fn from_with_config(value: T, cfg: &PoxConfig) -> Result<Self, Error>;
}
