#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchResult},
	pallet_prelude::*,
	PalletId,
};
use orml_traits::MultiCurrency;
use sp_arithmetic::traits::{checked_pow, One, Zero};
use sp_core::U256;
use sp_runtime::traits::{AccountIdConversion, StaticLookup};

pub use pallet::*;
mod primitives;
mod vault_asset;

use primitives::*;
use vault_asset::*;

#[allow(type_alias_bounds)]
type AccountIdOf<T: Config> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type AssetId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + TypeInfo + MaxEncodedLen;

		/// The trait control all currencies
		type MultiAsset: MultiCurrency<AccountIdOf<Self>, CurrencyId = Self::AssetId, Balance = Balance>;

		type VaultAssetGenerate: VaultAssetGenerator<Self::AssetId>;
		/// This pallet ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// vxasset -> origin asset
	#[pallet::storage]
	#[pallet::getter(fn vault_asset)]
	pub type Assets<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetId>;

	#[pallet::storage]
	#[pallet::getter(fn asset_metadata)]
	pub type AssetsMeta<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, AssetMeta>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		OriginAssetExisted,
		VaultAssetExisted,
		VaultAlreadyCreated,
		Math,
		ExceedMaxDeposit,
		ExceedMaxMint,
		ExceedMaxWithdraw,
		ExceedMaxRedeem,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		pub fn create_vault_asset(
			origin: OriginFor<T>,
			origin_asset: T::AssetId,
			origin_asset_decimal: u8,
			vault_asset_decimal: u8,
		) -> DispatchResult {
			ensure_root(origin)?;

			AssetsMeta::<T>::try_mutate_exists(origin_asset, |meta| -> DispatchResult {
				ensure!(meta.is_none(), Error::<T>::OriginAssetExisted);
				*meta = Some(AssetMeta {
					decimal: origin_asset_decimal,
				});
				Ok(())
			})?;

			let vault_asset = T::VaultAssetGenerate::generate(origin_asset);
			AssetsMeta::<T>::try_mutate_exists(vault_asset, |meta| -> DispatchResult {
				ensure!(meta.is_none(), Error::<T>::VaultAssetExisted);
				*meta = Some(AssetMeta {
					decimal: vault_asset_decimal,
				});
				Ok(())
			})?;

			Assets::<T>::try_mutate_exists(vault_asset, |optional_vault_asset| -> DispatchResult {
				ensure!(optional_vault_asset.is_none(), Error::<T>::VaultAlreadyCreated);
				*optional_vault_asset = Some(origin_asset);
				Ok(())
			})?;

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		pub fn deposit(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			recipient: <T::Lookup as StaticLookup>::Source,
			amounts: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(recipient)?;
			<Self as VaultAsset<T>>::deposit(&who, asset_id, amounts, &to)?;
			Ok(())
		}

		#[pallet::weight(1_000_000)]
		pub fn mint(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			recipient: <T::Lookup as StaticLookup>::Source,
			shares: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(recipient)?;
			<Self as VaultAsset<T>>::mint(&who, asset_id, shares, &to)?;
			Ok(())
		}

		#[pallet::weight(1_000_000)]
		pub fn withdraw(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			shares: Balance,
			recipient: <T::Lookup as StaticLookup>::Source,
			owner: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(recipient)?;
			let owner = T::Lookup::lookup(owner)?;
			<Self as VaultAsset<T>>::withdraw(&who, asset_id, shares, &to, &owner)?;
			Ok(())
		}

		#[pallet::weight(1_000_000)]
		pub fn redeem(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			shares: Balance,
			recipient: <T::Lookup as StaticLookup>::Source,
			owner: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(recipient)?;
			let owner = T::Lookup::lookup(owner)?;
			<Self as VaultAsset<T>>::redeem(&who, asset_id, shares, &to, &owner)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn asset_decimal(asset_id: T::AssetId) -> Result<u8, DispatchError> {
		Self::asset_metadata(asset_id)
			.map(|meta| meta.decimal)
			.ok_or(Error::<T>::VaultAssetExisted.into())
	}

	pub fn deposit_impl(
		asset_id: T::AssetId,
		who: &T::AccountId,
		receiver: &T::AccountId,
		amounts: Balance,
		shares: Balance,
	) -> DispatchResult {
		let origin_asset = Self::asset(asset_id)?;
		let pallet_account = T::PalletId::get().into_account_truncating();

		T::MultiAsset::transfer(origin_asset, who, &pallet_account, amounts)?;
		T::MultiAsset::deposit(asset_id, receiver, shares)
	}

	pub fn withdraw_impl(
		asset_id: T::AssetId,
		owner: &T::AccountId,
		receiver: &T::AccountId,
		amounts: Balance,
		shares: Balance,
	) -> DispatchResult {
		T::MultiAsset::withdraw(asset_id, owner, shares)?;
		let origin_asset = Self::asset(asset_id)?;
		let pallet_account = T::PalletId::get().into_account_truncating();

		T::MultiAsset::transfer(origin_asset, &pallet_account, receiver, amounts)
	}

	pub fn withdraw_fee_ratio(asset_id: T::AssetId) -> Balance {
		todo!()
	}

	pub fn calculate_withdraw_amounts(asset_id: T::AssetId, amounts: Balance) -> Option<(Balance, Balance)> {
		let fee_ratio = Self::withdraw_fee_ratio(asset_id);
		let withdraw_fee_amount = balance_mul_div(amounts, fee_ratio, 1e18 as Balance)?;
		let withdraw_amount = amounts.checked_sub(withdraw_fee_amount)?;
		Some((withdraw_amount, withdraw_fee_amount))
	}

	// pub fn burn_imp(asset_id: T::AssetId, receiver: &T::AccountId, shares: Balance) {}
}

impl<T: Config> VaultAsset<T> for Pallet<T> {
	fn asset(asset_id: T::AssetId) -> Result<T::AssetId, DispatchError> {
		Self::vault_asset(asset_id).ok_or(Error::<T>::VaultAssetExisted.into())
	}

	fn total_assets(asset_id: T::AssetId) -> Result<Balance, DispatchError> {
		let pallet_account = T::PalletId::get().into_account_truncating();
		let origin_asset = Self::asset(asset_id)?;
		Ok(T::MultiAsset::free_balance(origin_asset, &pallet_account))
	}

	fn convert_to_shares(asset_id: T::AssetId, amounts: Balance) -> Result<Balance, DispatchError> {
		let total_supply = T::MultiAsset::total_issuance(asset_id);
		if amounts == Zero::zero() || total_supply == Zero::zero() {
			let origin_asset = Self::asset(asset_id)?;
			let vault_asset_decimal = Self::asset_decimal(asset_id)?;
			let origin_asset_decimal = Self::asset_decimal(origin_asset)?;

			let calculate_fn = || {
				balance_mul_div(
					amounts,
					checked_pow(10, vault_asset_decimal as usize)?,
					checked_pow(10, origin_asset_decimal as usize)?,
				)
			};

			calculate_fn().ok_or(Error::<T>::Math.into())
		} else {
			balance_mul_div(amounts, total_supply, Self::total_assets(asset_id)?).ok_or(Error::<T>::Math.into())
		}
	}

	fn convert_to_assets(asset_id: T::AssetId, shares: Balance) -> Result<Balance, DispatchError> {
		let total_supply = T::MultiAsset::total_issuance(asset_id);
		if total_supply.is_zero() {
			let origin_asset = Self::asset(asset_id)?;
			let vault_asset_decimal = Self::asset_decimal(asset_id)?;
			let origin_asset_decimal = Self::asset_decimal(origin_asset)?;

			let calculate_fn = || {
				balance_mul_div(
					shares,
					checked_pow(10, origin_asset_decimal as usize)?,
					checked_pow(10, vault_asset_decimal as usize)?,
				)
			};

			calculate_fn().ok_or(Error::<T>::Math.into())
		} else {
			balance_mul_div(shares, Self::total_assets(asset_id)?, total_supply).ok_or(Error::<T>::Math.into())
		}
	}

	fn max_deposit(asset_id: T::AssetId, _receiver: &T::AccountId) -> Result<Balance, DispatchError> {
		let total_supply = T::MultiAsset::total_issuance(asset_id);
		let total_asset = Self::total_assets(asset_id)?;
		if !total_asset.is_zero() || total_supply.is_zero() {
			Ok(Balance::MAX)
		} else {
			Ok(Zero::zero())
		}
	}

	fn preview_deposit(asset_id: T::AssetId, amounts: Balance) -> Result<Balance, DispatchError> {
		Self::convert_to_shares(asset_id, amounts)
	}

	fn max_mint(_asset_id: T::AssetId, _receiver: &T::AccountId) -> Balance {
		Balance::MAX
	}

	fn preview_mint(asset_id: T::AssetId, shares: Balance) -> Result<Balance, DispatchError> {
		Self::convert_to_assets(asset_id, shares).and_then(|n| n.checked_add(One::one()).ok_or(Error::<T>::Math.into()))
	}

	fn max_withdraw(asset_id: T::AssetId, owner: &T::AccountId) -> Result<Balance, DispatchError> {
		Self::convert_to_assets(asset_id, T::MultiAsset::free_balance(asset_id, owner))
	}

	fn preview_withdraw(asset_id: T::AssetId, amounts: Balance) -> Result<Balance, DispatchError> {
		Self::convert_to_shares(asset_id, amounts)
			.and_then(|n| n.checked_add(One::one()).ok_or(Error::<T>::Math.into()))
	}

	fn max_redeem(asset_id: T::AssetId, owner: &T::AccountId) -> Balance {
		T::MultiAsset::free_balance(asset_id, owner)
	}

	fn preview_redeem(asset_id: T::AssetId, shares: Balance) -> Result<Balance, DispatchError> {
		Self::convert_to_assets(asset_id, shares)
	}

	fn deposit(
		who: &T::AccountId,
		asset_id: T::AssetId,
		amounts: Balance,
		to: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		ensure!(amounts < Self::max_deposit(asset_id, to)?, Error::<T>::ExceedMaxDeposit);

		let shares = Self::preview_deposit(asset_id, amounts)?;
		Self::deposit_impl(asset_id, who, to, amounts, shares)?;

		Ok(shares)
	}

	fn mint(
		who: &T::AccountId,
		asset_id: T::AssetId,
		shares: Balance,
		to: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		ensure!(shares < Self::max_mint(asset_id, to), Error::<T>::ExceedMaxMint);

		let assets = Self::preview_mint(asset_id, shares)?;
		Self::deposit_impl(asset_id, who, to, assets, shares)?;

		Ok(assets)
	}

	fn withdraw(
		who: &T::AccountId,
		asset_id: T::AssetId,
		amounts: Balance,
		to: &T::AccountId,
		owner: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		ensure!(
			amounts < Self::max_withdraw(asset_id, owner)?,
			Error::<T>::ExceedMaxWithdraw
		);

		let shares = Self::preview_withdraw(asset_id, amounts)?;
		let (amounts, fee) = Self::calculate_withdraw_amounts(asset_id, amounts).ok_or(Error::<T>::Math)?;

		Self::withdraw_impl(asset_id, who, to, amounts, shares)?;

		Ok(shares)
	}

	fn redeem(
		who: &T::AccountId,
		asset_id: T::AssetId,
		shares: Balance,
		to: &T::AccountId,
		owner: &T::AccountId,
	) -> Result<Balance, DispatchError> {
		ensure!(
			shares < Self::max_redeem(asset_id, owner),
			Error::<T>::ExceedMaxWithdraw
		);
		let amounts = Self::preview_redeem(asset_id, shares)?;
		let (amounts, fee) = Self::calculate_withdraw_amounts(asset_id, amounts).ok_or(Error::<T>::Math)?;
		Self::withdraw_impl(asset_id, who, to, amounts, shares)?;

		Ok(amounts)
	}
}
