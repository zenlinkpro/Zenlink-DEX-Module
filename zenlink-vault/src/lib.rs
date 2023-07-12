// Copyright 2021-2022 Zenlink.
// Licensed under Apache 2.0.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

pub mod primitives;
mod vault;
pub mod weights;

pub use pallet::*;
pub use primitives::*;
use vault::*;

use sp_arithmetic::{
	traits::{checked_pow, Zero},
	Rounding,
};
use sp_runtime::traits::{AccountIdConversion, StaticLookup};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, PalletId};

use orml_traits::MultiCurrency;

pub use weights::WeightInfo;

#[allow(type_alias_bounds)]
type AccountIdOf<T: Config> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::transactional;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The id of asset.
		type AssetId: Parameter
			+ Member
			+ Copy
			+ MaybeSerializeDeserialize
			+ Ord
			+ TypeInfo
			+ MaxEncodedLen;

		/// The trait control all assets.
		type MultiAsset: MultiCurrency<
			AccountIdOf<Self>,
			CurrencyId = Self::AssetId,
			Balance = Balance,
		>;

		/// The Trait generate vault asset for specific asset.
		type VaultAssetGenerate: VaultAssetGenerate<Self::AssetId>;

		/// This pallet id.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// The holding of a specific vault asset for specific asset.
	#[pallet::storage]
	#[pallet::getter(fn vault_asset)]
	pub type Vaults<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetId>;

	/// Metadata of a asset.
	#[pallet::storage]
	#[pallet::getter(fn asset_metadata)]
	pub type AssetMetadata<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, Metadata<T::AssetId>>;

	/// Ratio of a vault asset. The key is the underlying asset of the vault asset.
	#[pallet::storage]
	#[pallet::getter(fn asset_ratio)]
	pub type VaultRatio<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, Ratio>;

	/// The set of locked accounts for specific asset.
	#[pallet::storage]
	#[pallet::getter(fn asset_locker)]
	pub type AssetLockedAccounts<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, BTreeSet<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreateVaultAsset {
			underlying_asset: T::AssetId,
			underlying_asset_decimal: u8,
			vault_asset: T::AssetId,
			vault_asset_decimal: u8,
		},
		AddAssetLockedAccount {
			underlying_asset: T::AssetId,
			locked_accounts: Vec<T::AccountId>,
		},
		RemoveAssetLockedAccount {
			underlying_asset: T::AssetId,
			locked_accounts: Vec<T::AccountId>,
		},
		UpdateMaxPenaltyPatio {
			vault_asset: T::AssetId,
			ratio: Balance,
		},
		UpdateMinPenaltyPatio {
			vault_asset: T::AssetId,
			ratio: Balance,
		},
		Withdraw {
			owner: T::AccountId,
			asset_id: T::AssetId,
			receiver: T::AccountId,
			amounts: Balance,
			fee: Balance,
			shares: Balance,
		},
		Deposit {
			caller: T::AccountId,
			underlying_asset_id: T::AssetId,
			vault_asset_id: T::AssetId,
			receiver: T::AccountId,
			amounts: Balance,
			shares: Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The error for generate vault asset for the underlying asset.
		UnderlyingAssetError,
		/// The underlying asset is already exist.
		UnderlyingAssetExisted,
		/// The specific underlying asset has not been processed.
		UnknownUnderlyingAsset,
		/// The vault asset has not been created.
		UnknownVaultAsset,
		/// The vault asset is already created.
		VaultAssetExisted,
		/// The error generate by math calculation.
		Math,
		/// Exceed the max deposit amount.
		ExceedMaxDeposit,
		/// Exceed the max mint amount.
		ExceedMaxMint,
		/// Exceed the max withdraw amount.
		ExceedMaxWithdraw,
		/// Exceed the max redeem amount.
		ExceedMaxRedeem,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::create_vault_asset())]
		#[transactional]
		pub fn create_vault_asset(
			origin: OriginFor<T>,
			underlying_asset_id: T::AssetId,
			underlying_asset_decimal: u8,
			vault_asset_decimal: u8,
			max_penalty_ratio: Balance,
			min_penalty_ratio: Balance,
		) -> DispatchResult {
			ensure_root(origin)?;

			let vault_asset_id = T::VaultAssetGenerate::generate(underlying_asset_id)
				.ok_or(Error::<T>::UnderlyingAssetError)?;

			Vaults::<T>::try_mutate_exists(
				underlying_asset_id,
				|optional_vault_asset| -> DispatchResult {
					ensure!(optional_vault_asset.is_none(), Error::<T>::VaultAssetExisted);
					*optional_vault_asset = Some(vault_asset_id);
					Ok(())
				},
			)?;

			AssetMetadata::<T>::try_mutate_exists(underlying_asset_id, |meta| -> DispatchResult {
				ensure!(meta.is_none(), Error::<T>::UnderlyingAssetExisted);
				*meta = Some(Metadata {
					related_asset_id: vault_asset_id,
					decimal: underlying_asset_decimal,
				});
				Ok(())
			})?;

			AssetMetadata::<T>::try_mutate_exists(vault_asset_id, |meta| -> DispatchResult {
				ensure!(meta.is_none(), Error::<T>::VaultAssetExisted);
				*meta = Some(Metadata {
					related_asset_id: underlying_asset_id,
					decimal: vault_asset_decimal,
				});
				Ok(())
			})?;

			VaultRatio::<T>::try_mutate_exists(underlying_asset_id, |ratio| -> DispatchResult {
				*ratio = Some(Ratio { max_penalty_ratio, min_penalty_ratio });
				Ok(())
			})?;

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		pub fn update_max_penalty_ratio(
			origin: OriginFor<T>,
			vault_asset: T::AssetId,
			ratio: Balance,
		) -> DispatchResult {
			ensure_root(origin)?;
			VaultRatio::<T>::try_mutate_exists(vault_asset, |ratios| -> DispatchResult {
				match ratios {
					None => Err(Error::<T>::UnknownVaultAsset.into()),
					Some(m) => {
						m.max_penalty_ratio = ratio;
						Self::deposit_event(Event::UpdateMaxPenaltyPatio { vault_asset, ratio });
						Ok(())
					},
				}
			})
		}

		#[pallet::weight(1_000_000)]
		pub fn update_min_penalty_ratio(
			origin: OriginFor<T>,
			vault_asset: T::AssetId,
			ratio: Balance,
		) -> DispatchResult {
			ensure_root(origin)?;

			VaultRatio::<T>::try_mutate_exists(vault_asset, |ratios| -> DispatchResult {
				match ratios {
					None => Err(Error::<T>::UnknownVaultAsset.into()),
					Some(m) => {
						m.min_penalty_ratio = ratio;

						Self::deposit_event(Event::UpdateMinPenaltyPatio { vault_asset, ratio });
						Ok(())
					},
				}
			})
		}

		#[pallet::weight(1_000_000)]
		pub fn add_asset_locked_accounts(
			origin: OriginFor<T>,
			underlying_asset: T::AssetId,
			accounts: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				Vaults::<T>::contains_key(underlying_asset),
				Error::<T>::UnknownUnderlyingAsset
			);

			AssetLockedAccounts::<T>::try_mutate(
				underlying_asset,
				|locked_account_set| -> DispatchResult {
					let _ =
						accounts.iter().map(|account| locked_account_set.insert(account.clone()));
					Ok(())
				},
			)
		}

		#[pallet::weight(1_000_000)]
		pub fn remove_asset_locked_accounts(
			origin: OriginFor<T>,
			underlying_asset: T::AssetId,
			accounts: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				Vaults::<T>::contains_key(underlying_asset),
				Error::<T>::UnknownUnderlyingAsset
			);

			AssetLockedAccounts::<T>::try_mutate(
				underlying_asset,
				|locked_account_set| -> DispatchResult {
					let _ = accounts.iter().map(|account| locked_account_set.remove(account));
					Ok(())
				},
			)
		}

		#[pallet::weight(T::WeightInfo::deposit())]
		pub fn deposit(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			amounts: Balance,
			recipient: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(recipient)?;
			<Self as Vault<T>>::deposit(&who, asset_id, amounts, &to)?;
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			shares: Balance,
			recipient: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(recipient)?;
			<Self as Vault<T>>::mint(&who, asset_id, shares, &to)?;
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::withdraw())]
		pub fn withdraw(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			shares: Balance,
			recipient: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(recipient)?;
			<Self as Vault<T>>::withdraw(&who, asset_id, shares, &to)?;
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::redeem())]
		pub fn redeem(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			shares: Balance,
			recipient: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(recipient)?;
			<Self as Vault<T>>::redeem(&who, asset_id, shares, &to)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn asset_decimal(asset_id: T::AssetId) -> Result<u8, DispatchError> {
		Self::asset_metadata(asset_id)
			.map(|meta| meta.decimal)
			.ok_or_else(|| Error::<T>::VaultAssetExisted.into())
	}

	fn deposit_impl(
		underlying_asset_id: T::AssetId,
		who: &T::AccountId,
		receiver: &T::AccountId,
		amounts: Balance,
		shares: Balance,
	) -> DispatchResult {
		let vault_asset_id = Self::asset(underlying_asset_id)?;
		let pallet_account = T::PalletId::get().into_account_truncating();

		T::MultiAsset::transfer(underlying_asset_id, who, &pallet_account, amounts)?;
		T::MultiAsset::deposit(vault_asset_id, receiver, shares)?;

		Self::deposit_event(Event::Deposit {
			caller: who.clone(),
			underlying_asset_id,
			vault_asset_id,
			receiver: receiver.clone(),
			amounts,
			shares,
		});

		Ok(())
	}

	fn withdraw_impl(
		underlying_asset_id: T::AssetId,
		owner: &T::AccountId,
		receiver: &T::AccountId,
		amounts: Balance,
		shares: Balance,
	) -> DispatchResult {
		let vault_asset_id = Self::asset(underlying_asset_id)?;

		T::MultiAsset::withdraw(vault_asset_id, owner, shares)?;
		let pallet_account = T::PalletId::get().into_account_truncating();

		T::MultiAsset::transfer(underlying_asset_id, &pallet_account, receiver, amounts)
	}

	fn withdraw_fee_ratio(asset_id: T::AssetId) -> Option<Balance> {
		let asset_circulation = Self::asset_circulation(asset_id)?;
		let pallet_account = T::PalletId::get().into_account_truncating();
		let reserve = T::MultiAsset::free_balance(asset_id, &pallet_account);

		let share = balance_mul_div(reserve, 1e18 as Balance, asset_circulation, Rounding::Down)?;
		let asset_meta = Self::asset_ratio(asset_id)?;
		if share < 1e17 as Balance {
			Some(asset_meta.max_penalty_ratio)
		} else if share > 5e17 as Balance {
			Some(asset_meta.min_penalty_ratio)
		} else {
			let step = balance_mul_div(
				asset_meta.max_penalty_ratio.checked_sub(asset_meta.min_penalty_ratio)?,
				1e18 as Balance,
				4e17 as Balance,
				Rounding::Down,
			)?;

			balance_mul_div(
				share.checked_sub(1e17 as Balance)?,
				step,
				1e18 as Balance,
				Rounding::Down,
			)
			.and_then(|n| asset_meta.max_penalty_ratio.checked_sub(n))
		}
	}

	fn calculate_withdraw_amounts(
		asset_id: T::AssetId,
		amounts: Balance,
	) -> Option<(Balance, Balance)> {
		let fee_ratio = Self::withdraw_fee_ratio(asset_id)?;
		let withdraw_fee_amount =
			balance_mul_div(amounts, fee_ratio, 1e18 as Balance, Rounding::Down)?;
		let withdraw_amount = amounts.checked_sub(withdraw_fee_amount)?;
		Some((withdraw_amount, withdraw_fee_amount))
	}

	fn asset_circulation(asset_id: T::AssetId) -> Option<Balance> {
		let mut total_supply = T::MultiAsset::total_issuance(asset_id);
		let locked_accounts = AssetLockedAccounts::<T>::get(asset_id);

		for account in locked_accounts.iter() {
			total_supply =
				total_supply.checked_sub(T::MultiAsset::free_balance(asset_id, account))?;
		}
		Some(total_supply)
	}

	fn vault_asset_total_supply(underlying_asset_id: T::AssetId) -> Result<Balance, DispatchError> {
		let underlying_asset = Self::asset(underlying_asset_id)?;
		let vault_asset_total_supply = T::MultiAsset::total_issuance(underlying_asset);
		Ok(vault_asset_total_supply)
	}

	fn convert_to_shares_impl(
		underlying_asset_id: T::AssetId,
		amounts: Balance,
		rounding: Rounding,
	) -> Result<Balance, DispatchError> {
		let vault_asset_total_supply = Self::vault_asset_total_supply(underlying_asset_id)?;
		if amounts == Zero::zero() || vault_asset_total_supply == Zero::zero() {
			let underlying_asset = Self::asset(underlying_asset_id)?;
			let vault_asset_decimal = Self::asset_decimal(underlying_asset_id)?;
			let underlying_asset_decimal = Self::asset_decimal(underlying_asset)?;

			let calculate_fn = || {
				balance_mul_div(
					amounts,
					checked_pow(10, vault_asset_decimal as usize)?,
					checked_pow(10, underlying_asset_decimal as usize)?,
					rounding,
				)
			};

			calculate_fn().ok_or_else(|| Error::<T>::Math.into())
		} else {
			balance_mul_div(
				amounts,
				vault_asset_total_supply,
				Self::total_assets(underlying_asset_id)?,
				rounding,
			)
			.ok_or_else(|| Error::<T>::Math.into())
		}
	}

	fn convert_to_assets_impl(
		underlying_asset_id: T::AssetId,
		shares: Balance,
		rounding: Rounding,
	) -> Result<Balance, DispatchError> {
		let vault_asset = Self::asset(underlying_asset_id)?;
		let vault_asset_total_supply = T::MultiAsset::total_issuance(vault_asset);
		if vault_asset_total_supply.is_zero() {
			let vault_asset_decimal = Self::asset_decimal(vault_asset)?;
			let underlying_asset_decimal = Self::asset_decimal(underlying_asset_id)?;

			let calculate_fn = || {
				balance_mul_div(
					shares,
					checked_pow(10, underlying_asset_decimal as usize)?,
					checked_pow(10, vault_asset_decimal as usize)?,
					rounding,
				)
			};

			calculate_fn().ok_or_else(|| Error::<T>::Math.into())
		} else {
			balance_mul_div(
				shares,
				Self::total_assets(underlying_asset_id)?,
				vault_asset_total_supply,
				rounding,
			)
			.ok_or_else(|| Error::<T>::Math.into())
		}
	}
}

impl<T: Config> Vault<T> for Pallet<T> {
	fn asset(underlying_asset_id: T::AssetId) -> Result<T::AssetId, DispatchError> {
		Self::vault_asset(underlying_asset_id).ok_or_else(|| Error::<T>::VaultAssetExisted.into())
	}

	fn total_assets(underlying_asset_id: T::AssetId) -> Result<Balance, DispatchError> {
		let pallet_account = T::PalletId::get().into_account_truncating();
		Ok(T::MultiAsset::free_balance(underlying_asset_id, &pallet_account))
	}

	fn convert_to_shares(
		underlying_asset_id: T::AssetId,
		amounts: Balance,
	) -> Result<Balance, DispatchError> {
		Self::convert_to_shares_impl(underlying_asset_id, amounts, Rounding::Down)
	}

	fn convert_to_assets(
		underlying_asset_id: T::AssetId,
		shares: Balance,
	) -> Result<Balance, DispatchError> {
		Self::convert_to_assets_impl(underlying_asset_id, shares, Rounding::Down)
	}

	fn max_deposit(
		underlying_asset_id: T::AssetId,
		_receiver: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		let vault_asset_total_supply = Self::vault_asset_total_supply(underlying_asset_id)?;
		let total_asset = Self::total_assets(underlying_asset_id)?;
		if !total_asset.is_zero() || vault_asset_total_supply.is_zero() {
			Ok(Balance::MAX)
		} else {
			Ok(Zero::zero())
		}
	}

	fn preview_deposit(
		underlying_asset_id: T::AssetId,
		amounts: Balance,
	) -> Result<Balance, DispatchError> {
		Self::convert_to_shares(underlying_asset_id, amounts)
	}

	fn max_mint(
		_underlying_asset_id: T::AssetId,
		_receiver: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		Ok(Balance::MAX)
	}

	fn preview_mint(
		underlying_asset_id: T::AssetId,
		shares: Balance,
	) -> Result<Balance, DispatchError> {
		Self::convert_to_assets_impl(underlying_asset_id, shares, Rounding::Up)
	}

	fn max_withdraw(
		underlying_asset_id: T::AssetId,
		owner: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		let vault_asset = Self::asset(underlying_asset_id)?;
		Self::convert_to_assets(
			underlying_asset_id,
			T::MultiAsset::free_balance(vault_asset, owner),
		)
	}

	fn preview_withdraw(
		underlying_asset_id: T::AssetId,
		amounts: Balance,
	) -> Result<Balance, DispatchError> {
		Self::convert_to_shares_impl(underlying_asset_id, amounts, Rounding::Up)
	}

	fn max_redeem(
		underlying_asset_id: T::AssetId,
		owner: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		let vault_asset_id = Self::asset(underlying_asset_id)?;
		Ok(T::MultiAsset::free_balance(vault_asset_id, owner))
	}

	fn preview_redeem(
		underlying_asset_id: T::AssetId,
		shares: Balance,
	) -> Result<Balance, DispatchError> {
		Self::convert_to_assets_impl(underlying_asset_id, shares, Rounding::Down)
	}

	fn deposit(
		who: &T::AccountId,
		underlying_asset_id: T::AssetId,
		amounts: Balance,
		to: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		ensure!(
			amounts < Self::max_deposit(underlying_asset_id, to)?,
			Error::<T>::ExceedMaxDeposit
		);

		let shares = Self::preview_deposit(underlying_asset_id, amounts)?;
		Self::deposit_impl(underlying_asset_id, who, to, amounts, shares)?;

		Ok(shares)
	}

	fn mint(
		who: &T::AccountId,
		underlying_asset_id: T::AssetId,
		shares: Balance,
		to: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		ensure!(shares < Self::max_mint(underlying_asset_id, to)?, Error::<T>::ExceedMaxMint);

		let assets = Self::preview_mint(underlying_asset_id, shares)?;
		Self::deposit_impl(underlying_asset_id, who, to, assets, shares)?;

		Ok(assets)
	}

	fn withdraw(
		who: &T::AccountId,
		underlying_asset_id: T::AssetId,
		amounts: Balance,
		to: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		ensure!(
			amounts <= Self::max_withdraw(underlying_asset_id, who)?,
			Error::<T>::ExceedMaxWithdraw
		);

		let shares = Self::preview_withdraw(underlying_asset_id, amounts)?;
		let (amounts, fee) = Self::calculate_withdraw_amounts(underlying_asset_id, amounts)
			.ok_or(Error::<T>::Math)?;
		Self::withdraw_impl(underlying_asset_id, who, to, amounts, shares)?;

		Self::deposit_event(Event::Withdraw {
			owner: who.clone(),
			asset_id: underlying_asset_id,
			receiver: to.clone(),
			amounts,
			fee,
			shares,
		});

		Ok(shares)
	}

	fn redeem(
		who: &T::AccountId,
		underlying_asset_id: T::AssetId,
		shares: Balance,
		to: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		ensure!(shares <= Self::max_redeem(underlying_asset_id, who)?, Error::<T>::ExceedMaxRedeem);
		let amounts = Self::preview_redeem(underlying_asset_id, shares)?;
		let (amounts, fee) = Self::calculate_withdraw_amounts(underlying_asset_id, amounts)
			.ok_or(Error::<T>::Math)?;
		Self::withdraw_impl(underlying_asset_id, who, to, amounts, shares)?;

		Self::deposit_event(Event::Withdraw {
			owner: who.clone(),
			asset_id: underlying_asset_id,
			receiver: to.clone(),
			amounts,
			fee,
			shares,
		});

		Ok(amounts)
	}
}
