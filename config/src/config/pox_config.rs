use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct PoxConfig {
    pub cooridnate_precision_bigint: u32,
    pub rspr_precision_bigint: u32,

    pub penalty: PenaltyConfig,
    pub kernel: KernelConfig,

    pub pod_max_value: Decimal,
}
impl Default for PoxConfig {
    fn default() -> Self {
        Self {
            cooridnate_precision_bigint: 3,
            rspr_precision_bigint: 4,
            penalty: PenaltyConfig { max_diff: dec!(10) },
            kernel: KernelConfig {
                gaussian: GaussianConfig {
                    sigma: dec!(40000),
                    vanilla: GaussianVanillaConfig { use_coef: false },
                    tylor: GaussianTylorConfig {
                        sigma_range: dec!(3.0),
                    },
                },
                quadratic: QuadraticConfig {
                    max_dis_sqr: dec!(10000),
                },
            },
            pod_max_value: dec!(-100),
        }
    }
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct PenaltyConfig {
    pub max_diff: Decimal,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct KernelConfig {
    pub gaussian: GaussianConfig,
    pub quadratic: QuadraticConfig,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct GaussianTylorConfig {
    pub sigma_range: Decimal,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct GaussianVanillaConfig {
    pub use_coef: bool,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct GaussianConfig {
    pub sigma: Decimal,
    pub vanilla: GaussianVanillaConfig,
    pub tylor: GaussianTylorConfig,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct QuadraticConfig {
    pub max_dis_sqr: Decimal,
}
