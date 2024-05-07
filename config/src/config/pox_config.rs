use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub enum KernelTypeConfig {
    GaussianTaylor,
    Quadratic,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct PoxConfig {
    pub coordinate_precision_bigint: u32,
    pub rspr_precision_bigint: u32,

    pub penalty: PenaltyConfig,
    pub kernel: KernelConfig,

    pub pod_max_value: Decimal,
}
impl PoxConfig {
    pub fn coordinate_precision_pow10(&self) -> u64 {
        10_u64.pow(self.coordinate_precision_bigint)
    }
    pub fn rspr_precision_pow10(&self) -> u64 {
        10_u64.pow(self.rspr_precision_bigint)
    }
}
impl Default for PoxConfig {
    fn default() -> Self {
        Self {
            coordinate_precision_bigint: 3,
            rspr_precision_bigint: 4,
            penalty: PenaltyConfig { max_diff: dec!(10) },
            kernel: KernelConfig {
                kernel_type: KernelTypeConfig::GaussianTaylor,
                // kernel_type: KernelTypeConfig::Quadratic,
                gaussian: GaussianConfig {
                    sigma: dec!(500),
                    vanilla: GaussianVanillaConfig { use_coef: false },
                    taylor: GaussianTaylorConfig {
                        max_order: 20,
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
    pub kernel_type: KernelTypeConfig,
    pub gaussian: GaussianConfig,
    pub quadratic: QuadraticConfig,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct GaussianTaylorConfig {
    pub max_order: usize,
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
    pub taylor: GaussianTaylorConfig,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
#[serde(deny_unknown_fields)]
pub struct QuadraticConfig {
    pub max_dis_sqr: Decimal,
}
