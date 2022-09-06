use super::*;
use sp_core::U256;

pub type Balance = u128;

pub fn balance_mul_div(x: Balance, y: Balance, z: Balance) -> Option<Balance> {
	U256::from(x)
		.checked_mul(U256::from(y))
		.and_then(|n| n.checked_div(U256::from(z)))
		.and_then(|n| TryInto::<Balance>::try_into(n).ok())
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
