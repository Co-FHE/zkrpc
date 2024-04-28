use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PoxConfig {
    pub cooridnate_precision_bigint: u32,
    pub rspr_precision_bigint: u32,
    pub sigma_range: Decimal,
    pub sigma: Decimal,
}
impl Default for PoxConfig {
    fn default() -> Self {
        Self {
            cooridnate_precision_bigint: 3,
            rspr_precision_bigint: 4,
            sigma_range: dec!(3.0),
            sigma: dec!(40_000),
        }
    }
}
