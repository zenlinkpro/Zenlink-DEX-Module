use super::*;
use sp_arithmetic::{helpers_128bit::*, Rounding};

pub type Balance = u128;

pub fn balance_mul_div(x: Balance, y: Balance, z: Balance, rounding: Rounding) -> Option<Balance> {
	multiply_by_rational_with_rounding(x, y, z, rounding)
}

/// The metadata about asset.
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct Metadata<AssetId> {
	pub related_asset_id: AssetId,
	pub decimal: u8,
}

/// The metadata about a vault asset.
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct Ratio {
	pub max_penalty_ratio: Balance,
	pub min_penalty_ratio: Balance,
}

pub trait VaultAssetGenerate<CurrencyId> {
	fn generate(asset: CurrencyId) -> Option<CurrencyId>;
}
