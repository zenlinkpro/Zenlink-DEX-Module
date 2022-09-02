use super::*;
pub type Balance = u128;

pub fn balance_mul_div(x: Balance, y: Balance, z: Balance) -> Option<Balance> {
	U256::from(x)
		.checked_mul(U256::from(y))
		.and_then(|n| n.checked_div(U256::from(z)))
		.and_then(|n| TryInto::<Balance>::try_into(n).ok())
}

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct AssetMeta {
	pub(crate) decimal: u8,
}

pub trait VaultAssetGenerator<CurrencyId> {
	fn generate(asset: CurrencyId) -> CurrencyId;
}
