// Copyright 2021-2022 Zenlink.
// Licensed under Apache 2.0.

use super::*;

pub struct ZenlinkMultiAssets<T, Native = (), Local = (), Other = ()>(
	PhantomData<(T, Native, Local, Other)>,
);

impl<T: Config<AssetId = AssetId>, NativeCurrency, Local, Other> MultiCurrency<T::AccountId>
	for ZenlinkMultiAssets<Pallet<T>, NativeCurrency, Local, Other>
where
	NativeCurrency: Currency<T::AccountId>,
	Local: MultiCurrency<T::AccountId, Balance = AssetBalance, CurrencyId = AssetId>,
	Other: MultiCurrency<T::AccountId, Balance = AssetBalance, CurrencyId = AssetId>,
{
	type Balance = AssetBalance;
	type CurrencyId = AssetId;

	fn minimum_balance(_asset_id: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}

	fn total_issuance(asset_id: Self::CurrencyId) -> Self::Balance {
		let self_chain_id: u32 = T::SelfParaId::get();
		match asset_id.asset_type {
			NATIVE if asset_id.is_native(T::SelfParaId::get()) =>
				NativeCurrency::total_issuance().saturated_into::<AssetBalance>(),
			LOCAL | LIQUIDITY if asset_id.chain_id == self_chain_id =>
				Local::total_issuance(asset_id).saturated_into::<AssetBalance>(),
			RESERVED if asset_id.chain_id == self_chain_id =>
				Other::total_issuance(asset_id).saturated_into::<AssetBalance>(),
			_ if asset_id.is_foreign(self_chain_id) => Pallet::<T>::foreign_total_supply(asset_id),
			_ => Default::default(),
		}
	}

	fn total_balance(asset_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		let self_chain_id: u32 = T::SelfParaId::get();
		match asset_id.asset_type {
			NATIVE if asset_id.is_native(self_chain_id) =>
				NativeCurrency::total_balance(who).saturated_into::<AssetBalance>(),
			LOCAL | LIQUIDITY if asset_id.chain_id == self_chain_id =>
				Local::total_balance(asset_id, who).saturated_into::<AssetBalance>(),
			RESERVED if asset_id.chain_id == self_chain_id =>
				Other::total_balance(asset_id, who).saturated_into::<AssetBalance>(),
			_ if asset_id.is_foreign(self_chain_id) =>
				Pallet::<T>::foreign_balance_of(asset_id, who),
			_ => Default::default(),
		}
	}

	fn free_balance(asset_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		let self_chain_id: u32 = T::SelfParaId::get();
		match asset_id.asset_type {
			NATIVE if asset_id.is_native(self_chain_id) =>
				NativeCurrency::free_balance(who).saturated_into::<AssetBalance>(),
			LOCAL | LIQUIDITY if asset_id.chain_id == self_chain_id =>
				Local::free_balance(asset_id, who).saturated_into::<AssetBalance>(),
			RESERVED if asset_id.chain_id == self_chain_id =>
				Other::free_balance(asset_id, who).saturated_into::<AssetBalance>(),
			_ if asset_id.is_foreign(self_chain_id) =>
				Pallet::<T>::foreign_balance_of(asset_id, who),
			_ => Default::default(),
		}
	}

	fn ensure_can_withdraw(
		_asset_id: Self::CurrencyId,
		_who: &T::AccountId,
		_amount: Self::Balance,
	) -> DispatchResult {
		Ok(())
	}

	fn transfer(
		asset_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		let self_chain_id: u32 = T::SelfParaId::get();
		match asset_id.asset_type {
			NATIVE if asset_id.is_native(T::SelfParaId::get()) => {
				let balance_amount = amount
					.try_into()
					.map_err(|_| DispatchError::Other("AmountToBalanceConversionFailed"))?;

				NativeCurrency::transfer(from, to, balance_amount, KeepAlive)
			},
			LOCAL | LIQUIDITY if asset_id.chain_id == self_chain_id =>
				Local::transfer(asset_id, from, to, amount),
			RESERVED if asset_id.chain_id == self_chain_id =>
				Other::transfer(asset_id, from, to, amount),
			_ if asset_id.is_foreign(T::SelfParaId::get()) =>
				Pallet::<T>::foreign_transfer(asset_id, from, to, amount),
			_ => Err(Error::<T>::UnsupportedAssetType.into()),
		}?;
		Ok(())
	}

	fn deposit(
		asset_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		let self_chain_id: u32 = T::SelfParaId::get();
		match asset_id.asset_type {
			NATIVE if asset_id.is_native(T::SelfParaId::get()) => {
				let balance_amount = amount
					.try_into()
					.map_err(|_| DispatchError::Other("AmountToBalanceConversionFailed"))?;

				let _ = NativeCurrency::deposit_creating(who, balance_amount);

				Ok(())
			},
			LOCAL | LIQUIDITY if asset_id.chain_id == self_chain_id =>
				Local::deposit(asset_id, who, amount),
			RESERVED if asset_id.chain_id == self_chain_id => Other::deposit(asset_id, who, amount),
			_ if asset_id.is_foreign(T::SelfParaId::get()) =>
				Pallet::<T>::foreign_mint(asset_id, who, amount),
			_ => Err(Error::<T>::UnsupportedAssetType.into()),
		}?;
		Ok(())
	}

	fn withdraw(
		asset_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		let self_chain_id: u32 = T::SelfParaId::get();
		match asset_id.asset_type {
			NATIVE if asset_id.is_native(self_chain_id) => {
				let balance_amount = amount
					.try_into()
					.map_err(|_| DispatchError::Other("AmountToBalanceConversionFailed"))?;

				let _ = NativeCurrency::withdraw(
					who,
					balance_amount,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?;
				Ok(())
			},
			LOCAL | LIQUIDITY if asset_id.chain_id == self_chain_id =>
				Local::withdraw(asset_id, who, amount),
			RESERVED if asset_id.chain_id == self_chain_id =>
				Other::withdraw(asset_id, who, amount),
			_ if asset_id.is_foreign(T::SelfParaId::get()) =>
				Pallet::<T>::foreign_burn(asset_id, who, amount),
			_ => Err(Error::<T>::UnsupportedAssetType.into()),
		}?;
		Ok(())
	}

	fn can_slash(_currency_id: Self::CurrencyId, _who: &T::AccountId, _value: Self::Balance) -> bool {
		false
	}

	fn slash(
		_currency_id: Self::CurrencyId,
		_who: &T::AccountId,
		_amount: Self::Balance,
	) -> Self::Balance {
		Default::default()
	}
}
