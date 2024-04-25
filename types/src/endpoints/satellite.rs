use crate::{endpoints::terminal::Terminal, FixedPoint};
use std::collections::HashMap;
pub struct Satellite<T: FixedPoint> {
    pub address: String,
    pub terminals: HashMap<String, Terminal<T>>,
}
