use super::*;

pub trait VaultAsset<T: Config> {
	fn asset(asset_id: T::AssetId) -> Result<T::AssetId, DispatchError>;

	fn total_assets(asset_id: T::AssetId) -> Result<Balance, DispatchError>;

	fn convert_to_shares(asset_id: T::AssetId, amounts: Balance) -> Result<Balance, DispatchError>;

	fn convert_to_assets(asset_id: T::AssetId, shares: Balance) -> Result<Balance, DispatchError>;

	fn max_deposit(asset_id: T::AssetId, receiver: &T::AccountId) -> Result<Balance, DispatchError>;

	fn preview_deposit(asset_id: T::AssetId, amounts: Balance) -> Result<Balance, DispatchError>;

	fn max_mint(asset_id: T::AssetId, receiver: &T::AccountId) -> Balance;

	fn preview_mint(asset_id: T::AssetId, shares: Balance) -> Result<Balance, DispatchError>;

	fn max_withdraw(asset_id: T::AssetId, owner: &T::AccountId) -> Result<Balance, DispatchError>;

	fn preview_withdraw(asset_id: T::AssetId, amounts: Balance) -> Result<Balance, DispatchError>;

	fn max_redeem(asset_id: T::AssetId, owner: &T::AccountId) -> Balance;

	fn preview_redeem(asset_id: T::AssetId, shares: Balance) -> Result<Balance, DispatchError>;

	fn deposit(
		who: &T::AccountId,
		asset_id: T::AssetId,
		amounts: Balance,
		to: &T::AccountId,
	) -> Result<Balance, DispatchError>;

	fn mint(
		who: &T::AccountId,
		asset_id: T::AssetId,
		shares: Balance,
		to: &T::AccountId,
	) -> Result<Balance, DispatchError>;

	fn withdraw(
		who: &T::AccountId,
		asset_id: T::AssetId,
		amounts: Balance,
		to: &T::AccountId,
		owner: &T::AccountId,
	) -> Result<Balance, DispatchError>;

	fn redeem(
		who: &T::AccountId,
		asset_id: T::AssetId,
		shares: Balance,
		to: &T::AccountId,
		owner: &T::AccountId,
	) -> Result<Balance, DispatchError>;
}
