use crate::{
	ensure,
	primitives::{AssetId, MultiAsset},
	sp_api_hidden_includes_decl_storage::hidden_include::{StorageMap, StorageValue},
	Assets, Balances, Config, DispatchResult, Error, Module,
	RawEvent::{Burned, Minted, Transferred},
	TokenBalance, TotalSupply, Vec,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// The main implementation block for the module.
impl<T: Config> Module<T> {
	/// public mutable functions

	/// Implement of the transfer function.
	pub(crate) fn inner_transfer(
		id: AssetId,
		owner: &T::AccountId,
		target: &T::AccountId,
		amount: TokenBalance,
	) -> DispatchResult {
		let owner_balance = <Balances<T>>::get((&id, owner));
		ensure!(
			owner_balance >= amount,
			Error::<T>::InsufficientAssetBalance
		);
		ensure!(id.is_para_currency(), Error::<T>::NotParaCurrency);

		let new_balance = owner_balance.saturating_sub(amount);

		<Balances<T>>::mutate((&id, owner), |balance| *balance = new_balance);
		<Balances<T>>::mutate((&id, target), |balance| {
			*balance = balance.saturating_add(amount)
		});

		Self::deposit_event(Transferred(id, owner.clone(), target.clone(), amount));

		Ok(())
	}

	/// Increase the total supply of the asset
	pub(crate) fn inner_mint(
		id: AssetId,
		owner: &T::AccountId,
		amount: TokenBalance,
	) -> DispatchResult {
		ensure!(id.is_para_currency(), Error::<T>::NotParaCurrency);
		let new_balance = <Balances<T>>::get((id, owner)).saturating_add(amount);

		// new asset
		if !<Assets>::get().contains(&id) {
			<Assets>::mutate(|list| list.push(id))
		}
		<Balances<T>>::mutate((id, owner), |balance| *balance = new_balance);
		<TotalSupply>::mutate(id, |supply| {
			*supply = supply.saturating_add(amount);
		});

		Self::deposit_event(Minted(id, owner.clone(), amount));

		Ok(())
	}

	/// Decrease the total supply of the asset
	pub(crate) fn inner_burn(
		id: AssetId,
		owner: &T::AccountId,
		amount: TokenBalance,
	) -> DispatchResult {
		ensure!(<Assets>::get().contains(&id), Error::<T>::AssetNotExists);
		ensure!(id.is_para_currency(), Error::<T>::NotParaCurrency);
		let new_balance = <Balances<T>>::get((id, owner))
			.checked_sub(amount)
			.ok_or(Error::<T>::InsufficientAssetBalance)?;

		<Balances<T>>::mutate((id, owner), |balance| *balance = new_balance);
		<TotalSupply>::mutate(id, |supply| {
			*supply = supply.saturating_sub(amount);
		});

		Self::deposit_event(Burned(id, owner.clone(), amount));

		Ok(())
	}

	// Public immutable functions

	/// Get the asset `id` balance of `owner`.
	pub fn balance_of(id: AssetId, owner: &T::AccountId) -> TokenBalance {
		<Balances<T>>::get((id, owner))
	}

	/// Get the total supply of an asset `id`.
	pub fn total_supply(id: AssetId) -> TokenBalance {
		<TotalSupply>::get(id)
	}

	/// Get the assets list
	pub fn assets_list() -> Vec<AssetId> {
		<Assets>::get()
	}
}

impl<T: Config> MultiAsset<T::AccountId, TokenBalance> for Module<T> {
	fn total_supply(asset_id: AssetId) -> TokenBalance {
		Self::total_supply(asset_id)
	}

	fn balance_of(asset_id: AssetId, who: &T::AccountId) -> TokenBalance {
		Self::balance_of(asset_id, who)
	}

	fn transfer(
		asset_id: AssetId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: TokenBalance,
	) -> DispatchResult {
		Self::inner_transfer(asset_id, from, to, amount)
	}

	fn withdraw(asset_id: AssetId, who: &T::AccountId, amount: TokenBalance) -> DispatchResult {
		Self::inner_burn(asset_id, who, amount)
	}

	fn deposit(asset_id: AssetId, who: &T::AccountId, amount: TokenBalance) -> DispatchResult {
		Self::inner_mint(asset_id, who, amount)
	}
}
