// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

mod traits;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime, transactional, PalletId};
use sp_arithmetic::traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Zero};
use sp_runtime::traits::{AccountIdConversion, StaticLookup};

use orml_traits::MultiCurrency;

pub use pallet::*;

#[allow(type_alias_bounds)]
type AccountIdOf<T: Config> = <T as frame_system::Config>::AccountId;

const FEE_DENOMINATOR: u32 = 1e10f64 as u32;
const A_PRECISION: u32 = 100;
const MAX_ITERATION: u32 = 255; // the number of iterations to sum d and y
const POOL_TOKEN_COMMON_DECIMALS: u32 = 18;

/// Some thresholds when setting the pool
const DAY: u32 = 24 * 60 * 60;
const MIN_RAMP_TIME: u32 = DAY;
const MAX_A: u32 = 1e6f32 as u32;
const MAX_A_CHANGE: u32 = 10u32;
const MAX_ADMIN_FEE: u32 = 1e10f64 as u32;
const MAX_SWAP_FEE: u32 = 1e8f64 as u32;

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct Pool<CurrencyId, Balance, AccountId> {
	pub pooled_currency_ids: Vec<CurrencyId>,
	pub lp_currency_id: CurrencyId,
	/// token i multiplier to reach POOL_TOKEN_COMMON_DECIMALS
	pub token_multipliers: Vec<Balance>,
	pub balances: Vec<Balance>,
	pub fee: Balance,
	pub admin_fee: Balance,
	pub initial_a: Balance,
	pub future_a: Balance,
	pub initial_a_time: Balance,
	pub future_a_time: Balance,
	pub pool_account: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::traits::traits::ValidateCurrency;
	use frame_support::dispatch::{Codec, DispatchResult};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + TypeInfo + MaxEncodedLen;

		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo
			+ From<usize>
			+ From<u64>
			+ From<u32>
			+ Into<u32>;

		type MultiCurrency: MultiCurrency<AccountIdOf<Self>, CurrencyId = Self::CurrencyId, Balance = Self::Balance>;

		type PoolId: Parameter + Codec + Copy + Ord + AtLeast32BitUnsigned + Zero + One + Default;

		type EnsurePoolAsset: ValidateCurrency<Self::CurrencyId>;

		type TimeProvider: UnixTime;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_pool_id)]
	pub type NextPoolId<T: Config> = StorageValue<_, T::PoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> =
		StorageMap<_, Blake2_128Concat, T::PoolId, Pool<T::CurrencyId, T::Balance, T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn fee_receiver)]
	pub type FeeReceiver<T: Config> = StorageValue<_, Option<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreatePool(T::PoolId, Vec<T::CurrencyId>, T::CurrencyId, T::Balance, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidPooledCurrency,
		InvalidLpCurrency,
		InvalidParameter,
		InvalidCurrencyDecimal,
		InvalidPoolId,
		Arithmetic,
		Deadline,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn update_fee_receiver(
			origin: OriginFor<T>,
			fee_receiver: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			ensure_root(origin)?;
			let account = T::Lookup::lookup(fee_receiver)?;
			FeeReceiver::<T>::mutate(|receiver| (*receiver = Some(account)));
			Ok(())
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			currency_ids: Vec<T::CurrencyId>,
			currency_decimals: Vec<u32>,
			lp_currency_id: T::CurrencyId,
			a: T::Balance,
			fee: T::Balance,
			admin_fee: T::Balance,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				T::EnsurePoolAsset::validate_pooled_currency(&currency_ids),
				Error::<T>::InvalidPooledCurrency
			);
			ensure!(
				T::EnsurePoolAsset::validate_pool_lp_currency(lp_currency_id),
				Error::<T>::InvalidLpCurrency
			);

			ensure!(
				currency_ids.len() == currency_decimals.len(),
				Error::<T>::InvalidParameter
			);
			ensure!(a < T::Balance::from(MAX_A), Error::<T>::InvalidParameter);
			ensure!(fee < T::Balance::from(MAX_SWAP_FEE), Error::<T>::InvalidParameter);
			ensure!(
				admin_fee < T::Balance::from(MAX_ADMIN_FEE),
				Error::<T>::InvalidParameter
			);

			let mut rate = Vec::new();

			for (i, _) in currency_ids.iter().enumerate() {
				ensure!(
					currency_decimals[i] <= POOL_TOKEN_COMMON_DECIMALS,
					Error::<T>::InvalidCurrencyDecimal
				);
				let r = 10u32
					.checked_pow(POOL_TOKEN_COMMON_DECIMALS - currency_decimals[i])
					.ok_or(Error::<T>::Arithmetic)?
					.into();
				rate.push(r)
			}

			NextPoolId::<T>::try_mutate(|next_pool_id| -> DispatchResult {
				let pool_id = *next_pool_id;
				let pool_account = T::PalletId::get().into_sub_account(pool_id);

				Pools::<T>::try_mutate_exists(pool_id, |pool_info| -> DispatchResult {
					ensure!(pool_info.is_none(), Error::<T>::InvalidPoolId);

					frame_system::Pallet::<T>::inc_providers(&pool_account);
					let a_with_precision = a
						.checked_mul(&T::Balance::from(A_PRECISION))
						.ok_or(Error::<T>::Arithmetic)?;

					*pool_info = Some(Pool {
						pooled_currency_ids: currency_ids.clone(),
						lp_currency_id,
						token_multipliers: rate,
						balances: vec![Zero::zero(); currency_ids.len()],
						fee,
						admin_fee,
						initial_a: a_with_precision,
						future_a: a_with_precision,
						initial_a_time: Zero::zero(),
						future_a_time: Zero::zero(),
						pool_account: pool_account.clone(),
					});

					Ok(())
				})?;

				*next_pool_id = next_pool_id.checked_add(&One::one()).ok_or(Error::<T>::Arithmetic)?;

				Self::deposit_event(Event::CreatePool(
					pool_id,
					currency_ids,
					lp_currency_id,
					a,
					pool_account,
				));
				Ok(())
			})
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amounts: Vec<T::Balance>,
			min_mint_amount: T::Balance,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			Self::inner_add_liquidity(&who, pool_id, &amounts, min_mint_amount)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn inner_add_liquidity(
		who: &T::AccountId,
		pool_id: T::PoolId,
		amounts: &[T::Balance],
		min_mint_amount: T::Balance,
	) -> DispatchResult {
		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;

			ensure!(
				pool.pooled_currency_ids.len() == amounts.len(),
				Error::<T>::InvalidParameter
			);
			let fees = vec![Zero::zero(), pool.pooled_currency_ids.len()];

			let fee_per_token = Self::calculate_fee_per_token(&pool).ok_or(Error::<T>::Arithmetic)?;

			let lp_total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);

			let mut d0 = T::Balance::default();
			let amp = Self::get_a_precise(&pool).ok_or(Error::<T>::Arithmetic)?;
			if lp_total_supply > Zero::zero() {
				d0 = Self::get_d(
					&Self::xp(&pool.balances, &pool.token_multipliers).ok_or(Error::<T>::Arithmetic)?,
					amp,
				)
				.ok_or(Error::<T>::Arithmetic)?;
			}

			let new_balances = pool.balances.clone();
			

			Ok(())
		})
	}

	fn calculate_fee_per_token(pool: &Pool<T::CurrencyId, T::Balance, T::AccountId>) -> Option<T::Balance> {
		let n_pooled_currency = T::Balance::from(pool.pooled_currency_ids.len());

		pool.fee
			.checked_mul(&n_pooled_currency)?
			.checked_div(&T::Balance::from(4u32).checked_mul(&n_pooled_currency.checked_sub(&One::one())?)?)
	}

	fn get_a_precise(pool: &Pool<T::CurrencyId, T::Balance, T::AccountId>) -> Option<T::Balance> {
		let now = T::Balance::try_from(T::TimeProvider::now().as_secs()).ok()?;
		if now > pool.future_a_time {
			Some(pool.future_a);
		}

		if pool.future_a > pool.initial_a {
			return pool.initial_a.checked_add(
				&pool
					.future_a
					.checked_sub(&pool.initial_a)?
					.checked_mul(&now.checked_sub(&pool.initial_a_time)?)?
					.checked_div(&pool.future_a_time.checked_sub(&pool.initial_a_time)?)?,
			);
		}

		pool.initial_a.checked_sub(
			&pool
				.initial_a_time
				.checked_sub(&pool.future_a_time)?
				.checked_mul(&now.checked_sub(&pool.initial_a_time)?)?
				.checked_div(&pool.future_a_time.checked_sub(&pool.initial_a_time)?)?,
		)
	}

	fn xp(balances: &[T::Balance], rates: &[T::Balance]) -> Option<Vec<T::Balance>> {
		let mut normalized_res = Vec::new();
		for (i, _) in balances.iter().enumerate() {
			normalized_res.push(balances[i].checked_mul(&rates[i])?)
		}
		Some(normalized_res)
	}

	fn get_d(balances: &[T::Balance], amp: T::Balance) -> Option<T::Balance> {
		let n_currencies = T::Balance::from(balances.len());
		let sum = Self::sum_of(balances)?;
		if sum == T::Balance::default() {
			return Some(T::Balance::default());
		}

		let mut d_prev = T::Balance::default();
		let mut d = sum;
		let ann = amp.checked_div(&n_currencies)?;
		let a_precision = T::Balance::from(A_PRECISION);

		for i in 0..MAX_ITERATION {
			let mut d_p = d;
			for b in balances.iter() {
				d_p = d_p.checked_mul(&d)?.checked_div(&b.checked_mul(&n_currencies)?)?;
			}
			d_prev = d;
			let numerator = ann
				.checked_mul(&sum)?
				.checked_div(&a_precision)?
				.checked_add(&d_p.checked_mul(&n_currencies)?)?
				.checked_mul(&d)?;

			let denominator = ann
				.checked_sub(&a_precision)?
				.checked_mul(&d)?
				.checked_div(&a_precision)?
				.checked_add(&n_currencies.checked_add(&One::one())?)?
				.checked_mul(&d_p)?;

			d = numerator.checked_div(&denominator)?;

			if d > d_prev {
				if d - d_prev > One::one() {
					return Some(d);
				}
			} else {
				if d_prev - d > One::one() {
					return Some(d);
				}
			}
		}
		None
	}

	fn sum_of(balances: &[T::Balance]) -> Option<T::Balance> {
		let mut sum = T::Balance::default();
		for b in balances.iter() {
			sum = sum.checked_add(&b)?
		}
		Some(sum)
	}
}
