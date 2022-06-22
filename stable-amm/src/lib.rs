// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

pub mod rpc;
pub mod traits;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{Codec, DispatchResult},
	pallet_prelude::*,
	traits::UnixTime,
	transactional, PalletId,
};
use orml_traits::MultiCurrency;
use sp_arithmetic::traits::{checked_pow, AtLeast32BitUnsigned, CheckedAdd, One, Zero};
use sp_core::U256;
use sp_runtime::traits::{AccountIdConversion, StaticLookup};
use sp_std::{ops::Sub, vec, vec::Vec};

pub use pallet::*;

#[allow(type_alias_bounds)]
type AccountIdOf<T: Config> = <T as frame_system::Config>::AccountId;

type Balance = u128;
type Number = Balance;

const FEE_DENOMINATOR: Number = 10_000_000_000;

// protect from division loss when run approximation loop
const A_PRECISION: Number = 100;

// the number of iterations to sum d and y
const MAX_ITERATION: u32 = 255;
const POOL_TOKEN_COMMON_DECIMALS: u32 = 18;

const DAY: u32 = 86400;
const MIN_RAMP_TIME: u32 = DAY;

// max_a with precision
const MAX_A: Number = 1_000_000;
const MAX_A_CHANGE: u32 = 10;
const MAX_ADMIN_FEE: Number = 10_000_000_000;
const MAX_SWAP_FEE: Number = 100_000_000;

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct Pool<CurrencyId, AccountId, BoundString> {
	pub currency_ids: Vec<CurrencyId>,
	pub lp_currency_id: CurrencyId,
	// token i multiplier to reach POOL_TOKEN_COMMON_DECIMALS
	pub token_multipliers: Vec<Balance>,
	// effective balance which might different from token balance of the pool account because it hold admin fee as well
	pub balances: Vec<Balance>,
	// swap fee ratio. Change on any action which move balance state far from the ideal state
	pub fee: Number,
	// admin fee in ratio of swap fee.
	pub admin_fee: Number,
	// observation of A, multiplied with A_PRECISION
	pub initial_a: Number,
	pub future_a: Number,
	pub initial_a_time: Number,
	pub future_a_time: Number,
	// the pool's account
	pub account: AccountId,
	pub admin_fee_receiver: AccountId,
	pub lp_currency_symbol: BoundString,
	pub lp_currency_decimal: u8,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;
	use traits::ValidateCurrency;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency ID type
		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + TypeInfo + MaxEncodedLen;

		/// The trait control all currencies
		type MultiCurrency: MultiCurrency<AccountIdOf<Self>, CurrencyId = Self::CurrencyId, Balance = Balance>;

		/// The pool ID type
		type PoolId: Parameter + Codec + Copy + Ord + AtLeast32BitUnsigned + Zero + One + Default;

		/// The trait verify currency for some scenes.
		type EnsurePoolAsset: ValidateCurrency<Self::CurrencyId>;

		/// The trait get timestamp of chain.
		type TimeProvider: UnixTime;

		#[pallet::constant]
		type PoolCurrencySymbolLimit: Get<u32>;

		/// This pallet ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// The id of next pool
	#[pallet::storage]
	#[pallet::getter(fn next_pool_id)]
	pub type NextPoolId<T: Config> = StorageValue<_, T::PoolId, ValueQuery>;

	/// Info of a pool.
	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::PoolId,
		Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
	>;

	/// The pool id corresponding to lp currency
	#[pallet::storage]
	#[pallet::getter(fn lp_currencies)]
	pub type LpCurrencies<T: Config> = StorageMap<_, Blake2_128Concat, T::CurrencyId, T::PoolId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A pool was created.
		CreatePool {
			pool_id: T::PoolId,
			currency_ids: Vec<T::CurrencyId>,
			lp_currency_id: T::CurrencyId,
			a: Number,
			account: T::AccountId,
			admin_fee_receiver: T::AccountId,
		},
		/// A pool's admin_fee_receiver was updated.
		UpdateAdminFeeReceiver {
			pool_id: T::PoolId,
			admin_fee_receiver: T::AccountId,
		},
		/// Supply some liquidity to a pool.
		AddLiquidity {
			pool_id: T::PoolId,
			who: T::AccountId,
			supply_amounts: Vec<Balance>,
			fees: Vec<Balance>,
			new_d: Balance,
			mint_amount: Balance,
		},
		/// Swap a amounts of currency to get other.
		TokenExchange {
			pool_id: T::PoolId,
			who: T::AccountId,
			in_index: u32,
			in_amount: Balance,
			out_index: u32,
			out_amount: Balance,
		},
		/// Remove some liquidity from a pool.
		RemoveLiquidity {
			pool_id: T::PoolId,
			who: T::AccountId,
			amounts: Vec<Balance>,
			fees: Vec<Balance>,
			new_total_supply: Balance,
		},
		/// Remove some liquidity from a pool to get only one currency.
		RemoveLiquidityOneCurrency {
			pool_id: T::PoolId,
			who: T::AccountId,
			out_index: u32,
			burn_amount: Balance,
			out_amount: Balance,
		},
		/// Remove liquidity from a pool with specify the amounts of currencies to be obtained.
		RemoveLiquidityImbalance {
			pool_id: T::PoolId,
			who: T::AccountId,
			amounts: Vec<Balance>,
			fees: Vec<Balance>,
			new_d: Balance,
			new_total_supply: Balance,
		},
		/// A pool's fee parameters was updated
		NewFee {
			pool_id: T::PoolId,
			new_swap_fee: Number,
			new_admin_fee: Number,
		},
		/// A pool's 'A' was ramped.
		RampA {
			pool_id: T::PoolId,
			initial_a_precise: Number,
			future_a_precise: Number,
			now: Number,
			future_a_time: Number,
		},
		/// A pool's ramping A was stopped.
		StopRampA {
			pool_id: T::PoolId,
			current_a: Number,
			now: Number,
		},
		/// A pool's admin fee was collected.
		CollectProtocolFee {
			pool_id: T::PoolId,
			currency_id: T::CurrencyId,
			fee_amount: Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The currency id can't join stable amm pool.
		InvalidPooledCurrency,
		/// The currency id can't become the lp currency id of stable amm pool.
		InvalidLpCurrency,
		/// The parameters of a call are contradictory.
		MismatchParameter,
		/// The decimal of currency is invalid when create pool.
		InvalidCurrencyDecimal,
		/// The pool id is invalid.
		InvalidPoolId,
		/// The error generate by some arithmetic function.
		Arithmetic,
		/// The call already expired.
		Deadline,
		/// The caller does not have enough currencies.
		InsufficientSupply,
		/// The pool does not have enough currencies.
		InsufficientReserve,
		/// The new d below then older.
		CheckDFailed,
		/// Slippage is too large.
		AmountSlippage,
		/// Forbid swap same currency.
		SwapSameCurrency,
		/// The index of currency id bigger the length of pool's currencies;
		CurrencyIndexOutRange,
		/// The pool does not have enough lp currency.
		InsufficientLpReserve,
		/// The setting value exceed threshold.
		ExceedThreshold,
		/// The A of this pool is already ramped in current period.
		RampADelay,
		/// The value of feature_a_time is too small.
		MinRampTime,
		/// Forbid change A of a pool bigger than MAX_A.
		ExceedMaxAChange,
		/// The ramping A of this pool is already stopped.
		AlreadyStoppedRampA,
		/// The fee parameter exceed MAX_FEE when create pool.
		ExceedMaxFee,
		/// The admin fee parameter exceed MAX_ADMIN_FEE when create pool.
		ExceedMaxAdminFee,
		/// The A parameter exceed MAX_A when create pool.
		ExceedMaxA,
		/// The lp currency id is already used when create pool.
		LpCurrencyAlreadyUsed,
		/// Require all currencies of this pool when first supply.
		RequireAllCurrencies,
		/// The symbol of created pool maybe exceed length limit.
		BadPoolCurrencySymbol,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a stable amm pool.
		///
		/// Only admin can create pool.
		///
		/// # Argument
		///
		/// - `currency_ids`: The currencies will be join the created pool.
		/// - `currency_decimals`: The currencies corresponding decimals.
		/// - `lp_currency_id`: The specify lp currency id of the created pool.
		/// - `a`: The initial A of created pool.
		/// - `fee`: The swap fee of created pool.
		/// - `admin_fee`: The admin fee of created pool.
		/// - `admin_fee_receiver`: The admin fee receiver of created pool.
		/// - `lp_currency_symbol`: The symbol of created pool lp currency.
		/// - `lp_currency_decimal`: The decimal of created pool lp currency.
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			currency_ids: Vec<T::CurrencyId>,
			currency_decimals: Vec<u32>,
			lp_currency_id: T::CurrencyId,
			a: Number,
			fee: Number,
			admin_fee: Number,
			admin_fee_receiver: T::AccountId,
			lp_currency_symbol: Vec<u8>,
			lp_currency_decimal: u8,
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
				Self::lp_currencies(lp_currency_id).is_none(),
				Error::<T>::LpCurrencyAlreadyUsed
			);

			ensure!(
				currency_ids.len() == currency_decimals.len(),
				Error::<T>::MismatchParameter
			);
			ensure!(a < MAX_A, Error::<T>::ExceedMaxA);
			ensure!(fee <= MAX_SWAP_FEE, Error::<T>::ExceedMaxFee);
			ensure!(admin_fee <= MAX_ADMIN_FEE, Error::<T>::ExceedMaxAdminFee);

			let mut rate = Vec::new();

			for (i, _) in currency_ids.iter().enumerate() {
				ensure!(
					currency_decimals[i] <= POOL_TOKEN_COMMON_DECIMALS,
					Error::<T>::InvalidCurrencyDecimal
				);
				let r = checked_pow(
					Balance::from(10u32),
					(POOL_TOKEN_COMMON_DECIMALS - currency_decimals[i]) as usize,
				)
				.ok_or(Error::<T>::Arithmetic)?;
				rate.push(r)
			}

			NextPoolId::<T>::try_mutate(|next_pool_id| -> DispatchResult {
				let pool_id = *next_pool_id;
				let account = T::PalletId::get().into_sub_account(pool_id);

				Pools::<T>::try_mutate_exists(pool_id, |pool_info| -> DispatchResult {
					ensure!(pool_info.is_none(), Error::<T>::InvalidPoolId);

					frame_system::Pallet::<T>::inc_providers(&account);
					let a_with_precision = a.checked_mul(A_PRECISION).ok_or(Error::<T>::Arithmetic)?;

					let symbol: BoundedVec<u8, T::PoolCurrencySymbolLimit> = lp_currency_symbol
						.try_into()
						.map_err(|_| Error::<T>::BadPoolCurrencySymbol)?;

					*pool_info = Some(Pool {
						currency_ids: currency_ids.clone(),
						lp_currency_id,
						token_multipliers: rate,
						balances: vec![Zero::zero(); currency_ids.len()],
						fee,
						admin_fee,
						initial_a: a_with_precision,
						future_a: a_with_precision,
						initial_a_time: Zero::zero(),
						future_a_time: Zero::zero(),
						account: account.clone(),
						admin_fee_receiver: admin_fee_receiver.clone(),
						lp_currency_symbol: symbol,
						lp_currency_decimal,
					});

					Ok(())
				})?;

				LpCurrencies::<T>::insert(lp_currency_id, pool_id);

				*next_pool_id = next_pool_id.checked_add(&One::one()).ok_or(Error::<T>::Arithmetic)?;

				Self::deposit_event(Event::CreatePool {
					pool_id,
					currency_ids,
					lp_currency_id,
					a,
					account,
					admin_fee_receiver,
				});
				Ok(())
			})
		}

		/// Supply amounts of currencies to the pool.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		/// - `amounts`: Supply amounts of currencies.
		/// - `min_mint_amount`: The min amount of lp currency get.
		/// - `deadline`: Height of the cutoff block of this transaction
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amounts: Vec<Balance>,
			min_mint_amount: Balance,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			Self::inner_add_liquidity(&who, pool_id, &amounts, min_mint_amount)?;

			Ok(())
		}

		/// Swap a amounts of currencies to get other.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		/// - `from_index`: The index of swap currency id.
		/// - `to_index`: The index of receive currency id.
		/// - `in_amount`: The amounts of currencies swap.
		/// - `min_mint_amount`: The min amount of receive currency.
		/// - `deadline`: Height of the cutoff block of this transaction
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn swap(
			origin: OriginFor<T>,
			poo_id: T::PoolId,
			from_index: u32,
			to_index: u32,
			in_amount: Balance,
			min_out_amount: Balance,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			Self::inner_swap(
				&who,
				poo_id,
				from_index as usize,
				to_index as usize,
				in_amount,
				min_out_amount,
			)?;

			Ok(())
		}

		/// Remove liquidity from a pool.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		/// - `lp_amount`: The amounts of lp currency.
		/// - `min_amounts`: The min amounts of pool's currencies to get.
		/// - `deadline`: Height of the cutoff block of this transaction
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			poo_id: T::PoolId,
			lp_amount: Balance,
			min_amounts: Vec<Balance>,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			Self::inner_remove_liquidity(poo_id, &who, lp_amount, &min_amounts)?;

			Ok(())
		}

		/// Remove liquidity from a pool to get one currency.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		/// - `lp_amount`: The amounts of lp currency.
		/// - `index`: The index of receive currency.
		/// - `min_amount`: The min amounts of receive currency;
		/// - `deadline`: Height of the cutoff block of this transaction
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn remove_liquidity_one_currency(
			origin: OriginFor<T>,
			poo_id: T::PoolId,
			lp_amount: Balance,
			index: u32,
			min_amount: Balance,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			Self::inner_remove_liquidity_one_currency(poo_id, &who, lp_amount, index, min_amount)?;

			Ok(())
		}

		/// Remove liquidity from a pool to the specify amounts of currencies.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		/// - `amounts`: The specify amounts of receive currencies.
		/// - `max_burn_amount`: The max amount of burned lp currency.
		/// - `deadline`: Height of the cutoff block of this transaction
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn remove_liquidity_imbalance(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amounts: Vec<Balance>,
			max_burn_amount: Balance,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			Self::inner_remove_liquidity_imbalance(&who, pool_id, &amounts, max_burn_amount)?;

			Ok(())
		}

		/// Update admin fee receiver of the pool.
		///
		/// Only called by admin.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		/// - `fee_receiver`: The new admin fee receiver of this pool.
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn update_fee_receiver(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			fee_receiver: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			ensure_root(origin)?;
			let admin_fee_receiver = T::Lookup::lookup(fee_receiver)?;
			Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
				let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
				pool.admin_fee_receiver = admin_fee_receiver.clone();

				Self::deposit_event(Event::UpdateAdminFeeReceiver {
					pool_id,
					admin_fee_receiver,
				});
				Ok(())
			})
		}

		/// Update fee of the pool.
		///
		/// Only called by admin.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		/// - `new_swap_fee`: The new swap fee of this pool.
		/// - `new_admin_fee`: The new admin fee of this pool.
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn set_fee(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			new_swap_fee: Number,
			new_admin_fee: Number,
		) -> DispatchResult {
			ensure_root(origin)?;
			Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
				let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
				ensure!(new_swap_fee <= MAX_SWAP_FEE, Error::<T>::ExceedThreshold);
				ensure!(new_admin_fee <= MAX_ADMIN_FEE, Error::<T>::ExceedThreshold);

				pool.admin_fee = new_admin_fee;
				pool.fee = new_swap_fee;

				Self::deposit_event(Event::NewFee {
					pool_id,
					new_swap_fee,
					new_admin_fee,
				});
				Ok(())
			})
		}

		/// Start ramping up or down A parameter towards given future_a and future_a_time
		///
		/// Only called by admin.
		/// Checks if the change is too rapid, and commits the new A value only when it falls under
		/// the limit range.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		/// - `future_a`: The new A to ramp towards.
		/// - `future_a_time`: Timestamp when the new A should be reached
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn ramp_a(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			future_a: Number,
			future_a_time: Number,
		) -> DispatchResult {
			ensure_root(origin)?;
			let now = T::TimeProvider::now().as_secs() as Number;
			Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
				let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;

				ensure!(
					now >= pool
						.initial_a_time
						.checked_add(Number::from(DAY))
						.ok_or(Error::<T>::Arithmetic)?,
					Error::<T>::RampADelay
				);

				ensure!(
					future_a_time
						>= now
							.checked_add(Number::from(MIN_RAMP_TIME))
							.ok_or(Error::<T>::Arithmetic)?,
					Error::<T>::MinRampTime
				);

				ensure!(future_a > Zero::zero() && future_a < MAX_A, Error::<T>::ExceedThreshold);

				let (initial_a_precise, future_a_precise) = Self::get_a_precise(pool)
					.and_then(|initial_a_precise| -> Option<(Number, Number)> {
						let future_a_precise = future_a.checked_mul(A_PRECISION)?;
						Some((initial_a_precise, future_a_precise))
					})
					.ok_or(Error::<T>::Arithmetic)?;

				let max_a_change = Number::from(MAX_A_CHANGE);

				if future_a_precise < initial_a_precise {
					ensure!(
						future_a_precise
							.checked_mul(max_a_change)
							.ok_or(Error::<T>::Arithmetic)?
							>= initial_a_precise,
						Error::<T>::ExceedMaxAChange
					);
				} else {
					ensure!(
						future_a_precise
							<= initial_a_precise
								.checked_mul(max_a_change)
								.ok_or(Error::<T>::Arithmetic)?,
						Error::<T>::ExceedMaxAChange
					);
				}

				pool.initial_a = initial_a_precise;
				pool.future_a = future_a_precise;
				pool.initial_a_time = now;
				pool.future_a_time = future_a_time;

				Self::deposit_event(Event::RampA {
					pool_id,
					initial_a_precise,
					future_a_precise,
					now,
					future_a_time,
				});

				Ok(())
			})
		}

		/// Stop ramping A parameter.
		///
		/// Only called by admin.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn stop_ramp_a(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_root(origin)?;
			Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
				let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
				let now = T::TimeProvider::now().as_secs() as Number;
				ensure!(pool.future_a_time > now, Error::<T>::AlreadyStoppedRampA);

				let current_a = Self::get_a_precise(pool).ok_or(Error::<T>::Arithmetic)?;

				pool.initial_a = current_a;
				pool.future_a = current_a;
				pool.initial_a_time = now;
				pool.future_a_time = now;

				Self::deposit_event(Event::StopRampA {
					pool_id,
					current_a,
					now,
				});
				Ok(())
			})
		}

		/// Withdraw the admin fee from pool to admin fee receiver.
		///
		/// Only called by admin.
		///
		/// # Argument
		///
		/// - `pool_id`: The id of pool.
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn withdraw_admin_fee(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_root(origin)?;

			Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
				let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
				for (i, reserve) in pool.balances.iter().enumerate() {
					let balance = T::MultiCurrency::free_balance(pool.currency_ids[i], &pool.account)
						.checked_sub(*reserve)
						.ok_or(Error::<T>::Arithmetic)?;

					if !balance.is_zero() {
						T::MultiCurrency::transfer(
							pool.currency_ids[i],
							&pool.account,
							&pool.admin_fee_receiver,
							balance,
						)?;
					}
					Self::deposit_event(Event::CollectProtocolFee {
						pool_id,
						currency_id: pool.currency_ids[i],
						fee_amount: balance,
					});
				}
				Ok(())
			})
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn inner_add_liquidity(
		who: &T::AccountId,
		pool_id: T::PoolId,
		amounts: &[Balance],
		min_mint_amount: Balance,
	) -> DispatchResult {
		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;

			let n_currencies = pool.currency_ids.len();
			ensure!(n_currencies == amounts.len(), Error::<T>::MismatchParameter);
			let mut fees = Vec::new();
			let fee_per_token = Self::calculate_fee_per_token(pool).ok_or(Error::<T>::Arithmetic)?;

			let lp_total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);

			let mut d0 = Balance::default();
			let amp = Self::get_a_precise(pool).ok_or(Error::<T>::Arithmetic)?;
			if lp_total_supply > Zero::zero() {
				d0 = Self::get_d(
					&Self::xp(&pool.balances, &pool.token_multipliers).ok_or(Error::<T>::Arithmetic)?,
					amp,
				)
				.ok_or(Error::<T>::Arithmetic)?;
			}

			let mut new_balances = pool.balances.clone();

			for i in 0..n_currencies {
				if lp_total_supply == Zero::zero() {
					ensure!(!amounts[i].is_zero(), Error::<T>::RequireAllCurrencies);
				}
				new_balances[i] = new_balances[i]
					.checked_add(Self::do_transfer_in(
						pool.currency_ids[i],
						who,
						&pool.account,
						amounts[i],
					)?)
					.ok_or(Error::<T>::Arithmetic)?;
			}

			let mut d1 = Self::get_d(
				&Self::xp(&new_balances, &pool.token_multipliers).ok_or(Error::<T>::Arithmetic)?,
				amp,
			)
			.ok_or(Error::<T>::Arithmetic)?;

			ensure!(d1 > d0, Error::<T>::CheckDFailed);

			let mint_amount: Balance;
			if lp_total_supply.is_zero() {
				pool.balances = new_balances;
				mint_amount = d1;
			} else {
				(mint_amount, fees) = Self::calculate_mint_amount(
					pool,
					&mut new_balances,
					d0,
					&mut d1,
					fee_per_token,
					amp,
					lp_total_supply,
				)
				.ok_or(Error::<T>::Arithmetic)?;
			}

			ensure!(min_mint_amount <= mint_amount, Error::<T>::AmountSlippage);

			T::MultiCurrency::deposit(pool.lp_currency_id, who, mint_amount)?;

			Self::deposit_event(Event::AddLiquidity {
				pool_id,
				who: who.clone(),
				supply_amounts: amounts.to_vec(),
				fees: fees.to_vec(),
				new_d: d1,
				mint_amount,
			});
			Ok(())
		})
	}

	pub fn inner_swap(
		who: &T::AccountId,
		pool_id: T::PoolId,
		i: usize,
		j: usize,
		in_amount: Balance,
		out_min_amount: Balance,
	) -> DispatchResult {
		ensure!(i != j, Error::<T>::SwapSameCurrency);

		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
			let n_currencies = pool.currency_ids.len();
			ensure!(i < n_currencies && j < n_currencies, Error::<T>::CurrencyIndexOutRange);

			let in_amount = Self::do_transfer_in(pool.currency_ids[i], who, &pool.account, in_amount)?;

			let normalize_balances = Self::xp(&pool.balances, &pool.token_multipliers).ok_or(Error::<T>::Arithmetic)?;

			let x = in_amount
				.checked_mul(pool.token_multipliers[i])
				.and_then(|n| n.checked_add(normalize_balances[i]))
				.ok_or(Error::<T>::Arithmetic)?;

			let y = Self::get_y(pool, i, j, x, &normalize_balances).ok_or(Error::<T>::Arithmetic)?;

			let mut dy = normalize_balances[j]
				.checked_sub(y)
				.and_then(|n| n.checked_sub(One::one()))
				.ok_or(Error::<T>::Arithmetic)?;

			let dy_fee = U256::from(dy)
				.checked_mul(U256::from(pool.fee))
				.and_then(|n| n.checked_div(U256::from(FEE_DENOMINATOR)))
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())
				.ok_or(Error::<T>::Arithmetic)?;

			dy = dy
				.checked_sub(dy_fee)
				.and_then(|n| n.checked_div(pool.token_multipliers[j]))
				.ok_or(Error::<T>::Arithmetic)?;

			ensure!(dy >= out_min_amount, Error::<T>::AmountSlippage);

			let admin_fee = U256::from(dy_fee)
				.checked_mul(U256::from(pool.admin_fee))
				.and_then(|n| n.checked_div(U256::from(FEE_DENOMINATOR)))
				.and_then(|n| n.checked_div(U256::from(pool.token_multipliers[j])))
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())
				.ok_or(Error::<T>::Arithmetic)?;

			//update pool balance
			pool.balances[i] = pool.balances[i].checked_add(in_amount).ok_or(Error::<T>::Arithmetic)?;
			pool.balances[j] = pool.balances[j]
				.checked_sub(dy)
				.and_then(|n| n.checked_sub(admin_fee))
				.ok_or(Error::<T>::Arithmetic)?;

			T::MultiCurrency::transfer(pool.currency_ids[j], &pool.account, who, dy)
				.map_err(|_| Error::<T>::InsufficientReserve)?;

			Self::deposit_event(Event::TokenExchange {
				pool_id,
				who: who.clone(),
				in_index: i as u32,
				in_amount,
				out_index: j as u32,
				out_amount: dy,
			});

			Ok(())
		})
	}

	pub fn inner_remove_liquidity(
		pool_id: T::PoolId,
		who: &T::AccountId,
		lp_amount: Balance,
		min_amounts: &[Balance],
	) -> DispatchResult {
		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
			let lp_total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);

			ensure!(lp_total_supply >= lp_amount, Error::<T>::InsufficientReserve);

			let currencies_length = pool.currency_ids.len();
			let min_amounts_length = min_amounts.len();
			ensure!(currencies_length == min_amounts_length, Error::<T>::MismatchParameter);

			let fees: Vec<Balance> = vec![Zero::zero(); currencies_length];
			let amounts = Self::calculate_removed_liquidity(pool, lp_amount).ok_or(Error::<T>::Arithmetic)?;

			for (i, amount) in amounts.iter().enumerate() {
				ensure!(*amount >= min_amounts[i], Error::<T>::AmountSlippage);
				pool.balances[i] = pool.balances[i].checked_sub(*amount).ok_or(Error::<T>::Arithmetic)?;
				T::MultiCurrency::transfer(pool.currency_ids[i], &pool.account, who, *amount)?;
			}

			T::MultiCurrency::withdraw(pool.lp_currency_id, who, lp_amount)?;
			Self::deposit_event(Event::RemoveLiquidity {
				pool_id,
				who: who.clone(),
				amounts,
				fees,
				new_total_supply: lp_total_supply - lp_amount,
			});
			Ok(())
		})
	}

	pub fn inner_remove_liquidity_one_currency(
		pool_id: T::PoolId,
		who: &T::AccountId,
		lp_amount: Balance,
		index: u32,
		min_amount: Balance,
	) -> DispatchResult {
		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
			let total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);
			ensure!(total_supply > Zero::zero(), Error::<T>::InsufficientLpReserve);
			ensure!(
				T::MultiCurrency::free_balance(pool.lp_currency_id, who) >= lp_amount && lp_amount <= total_supply,
				Error::<T>::InsufficientSupply
			);
			ensure!(
				index < pool.currency_ids.len() as u32,
				Error::<T>::CurrencyIndexOutRange
			);

			let (dy, dy_fee) =
				Self::calculate_remove_liquidity_one_token(pool, lp_amount, index).ok_or(Error::<T>::Arithmetic)?;

			ensure!(dy >= min_amount, Error::<T>::AmountSlippage);
			let fee_denominator = U256::from(FEE_DENOMINATOR);

			pool.balances[index as usize] = U256::from(dy_fee)
				.checked_mul(U256::from(pool.admin_fee))
				.and_then(|n| n.checked_div(fee_denominator))
				.and_then(|n| n.checked_add(U256::from(dy)))
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())
				.and_then(|n| pool.balances[index as usize].checked_sub(n))
				.ok_or(Error::<T>::Arithmetic)?;

			T::MultiCurrency::withdraw(pool.lp_currency_id, who, lp_amount)?;
			T::MultiCurrency::transfer(pool.currency_ids[index as usize], &pool.account, who, dy)?;

			Self::deposit_event(Event::RemoveLiquidityOneCurrency {
				pool_id,
				who: who.clone(),
				out_index: index,
				burn_amount: lp_amount,
				out_amount: dy,
			});
			Ok(())
		})
	}

	pub fn inner_remove_liquidity_imbalance(
		who: &T::AccountId,
		pool_id: T::PoolId,
		amounts: &[Balance],
		max_burn_amount: Balance,
	) -> DispatchResult {
		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
			let total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);

			ensure!(total_supply > Zero::zero(), Error::<T>::InsufficientLpReserve);
			ensure!(amounts.len() == pool.currency_ids.len(), Error::<T>::MismatchParameter);

			let (burn_amount, fees, d1) = Self::calculate_remove_liquidity_imbalance(pool, amounts, total_supply)
				.ok_or(Error::<T>::Arithmetic)?;
			ensure!(
				burn_amount > Zero::zero() && burn_amount <= max_burn_amount,
				Error::<T>::AmountSlippage
			);

			T::MultiCurrency::withdraw(pool.lp_currency_id, who, burn_amount)?;

			for (i, balance) in amounts.iter().enumerate() {
				if *balance > Zero::zero() {
					T::MultiCurrency::transfer(pool.currency_ids[i], &pool.account, who, *balance)?;
				}
			}

			Self::deposit_event(Event::RemoveLiquidityImbalance {
				pool_id,
				who: who.clone(),
				amounts: amounts.to_vec(),
				fees,
				new_d: d1,
				new_total_supply: total_supply - burn_amount,
			});

			Ok(())
		})
	}

	fn calculate_fee_per_token(
		pool: &Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
	) -> Option<Balance> {
		let n_pooled_currency = Balance::from(pool.currency_ids.len() as u64);

		pool.fee
			.checked_mul(n_pooled_currency)?
			.checked_div(Balance::from(4u32).checked_mul(n_pooled_currency.checked_sub(One::one())?)?)
	}

	fn calculate_mint_amount(
		pool: &mut Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
		new_balances: &mut [Balance],
		d0: Balance,
		d1: &mut Balance,
		fee: Balance,
		amp: Balance,
		total_supply: Balance,
	) -> Option<(Balance, Vec<Balance>)> {
		let mut diff: U256;
		let n_currencies = pool.currency_ids.len();
		let fee_denominator = U256::from(FEE_DENOMINATOR);
		let mut fees = vec![Zero::zero(); n_currencies];

		for i in 0..n_currencies {
			diff = Self::distance(
				U256::from(*d1)
					.checked_mul(U256::from(pool.balances[i]))
					.and_then(|n| n.checked_div(U256::from(d0)))?,
				U256::from(new_balances[i]),
			);

			fees[i] = U256::from(fee)
				.checked_mul(diff)
				.and_then(|n| n.checked_div(fee_denominator))
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())?;

			pool.balances[i] = new_balances[i].checked_sub(
				U256::from(fees[i])
					.checked_mul(U256::from(pool.admin_fee))
					.and_then(|n| n.checked_div(fee_denominator))
					.and_then(|n| TryInto::<Balance>::try_into(n).ok())?,
			)?;

			new_balances[i] = new_balances[i].checked_sub(fees[i])?;
		}
		*d1 = Self::get_d(&Self::xp(new_balances, &pool.token_multipliers)?, amp)?;

		let mint_amount = U256::from(total_supply)
			.checked_mul(U256::from(*d1).checked_sub(U256::from(d0))?)?
			.checked_div(U256::from(d0))
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())?;

		Some((mint_amount, fees))
	}

	pub fn calculate_swap_amount(
		pool: &Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
		i: usize,
		j: usize,
		in_balance: Balance,
	) -> Option<Balance> {
		let n_currencies = pool.currency_ids.len();
		if i >= n_currencies || j >= n_currencies {
			return None;
		}

		let fee_denominator = FEE_DENOMINATOR;

		let normalize_balances = Self::xp(&pool.balances, &pool.token_multipliers)?;
		let new_in_balance = normalize_balances[i].checked_add(in_balance.checked_mul(pool.token_multipliers[i])?)?;

		let out_balance = Self::get_y(pool, i, j, new_in_balance, &normalize_balances)?;
		let mut out_amount = normalize_balances[j]
			.checked_sub(out_balance)?
			.checked_sub(One::one())?
			.checked_div(pool.token_multipliers[j])?;

		let fee = U256::from(out_amount)
			.checked_mul(U256::from(pool.fee))?
			.checked_div(U256::from(fee_denominator))
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())?;

		out_amount = out_amount.checked_sub(fee)?;

		Some(out_amount)
	}

	pub fn calculate_removed_liquidity(
		pool: &Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
		amount: Balance,
	) -> Option<Vec<Balance>> {
		let lp_total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);
		if lp_total_supply < amount {
			return None;
		}
		let mut amounts = Vec::new();
		for b in pool.balances.iter() {
			amounts.push(
				U256::from(*b)
					.checked_mul(U256::from(amount))?
					.checked_div(U256::from(lp_total_supply))
					.and_then(|n| TryInto::<Balance>::try_into(n).ok())?,
			);
		}
		Some(amounts)
	}

	pub fn calculate_remove_liquidity_one_token(
		pool: &Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
		token_amount: Balance,
		index: u32,
	) -> Option<(Balance, Balance)> {
		if index > pool.currency_ids.len() as u32 {
			return None;
		}
		let total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);

		let amp = Self::get_a_precise(pool)?;
		let mut xp = Self::xp(&pool.balances, &pool.token_multipliers)?;
		let d0 = Self::get_d(&pool.balances, amp)?;

		let d1 = U256::from(d0)
			.checked_sub(
				U256::from(token_amount)
					.checked_mul(U256::from(d0))?
					.checked_div(U256::from(total_supply))?,
			)
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())?;

		let new_y = Self::get_yd(pool, amp, index, &xp, d1)?;

		let fee_pre_token = U256::from(Self::calculate_fee_per_token(pool)?);
		let fee_denominator = U256::from(FEE_DENOMINATOR);

		for (i, x) in xp.clone().iter().enumerate() {
			let expected_dx = if i as u32 == index {
				U256::from(*x)
					.checked_mul(U256::from(d1))?
					.checked_div(U256::from(d0))?
					.checked_sub(U256::from(new_y))?
			} else {
				U256::from(*x).checked_sub(
					U256::from(*x)
						.checked_mul(U256::from(d1))?
						.checked_div(U256::from(d0))?,
				)?
			};
			xp[i] = xp[i].checked_sub(
				fee_pre_token
					.checked_mul(expected_dx)?
					.checked_div(fee_denominator)
					.and_then(|n| TryInto::<Balance>::try_into(n).ok())?,
			)?;
		}

		let mut dy = xp[index as usize].checked_sub(Self::get_yd(pool, amp, index, &xp, d1)?)?;
		dy = dy
			.checked_sub(One::one())?
			.checked_div(pool.token_multipliers[index as usize])?;

		let fee = xp[index as usize]
			.checked_sub(new_y)?
			.checked_div(pool.token_multipliers[index as usize])?
			.checked_sub(dy)?;

		Some((dy, fee))
	}

	fn calculate_remove_liquidity_imbalance(
		pool: &mut Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
		amounts: &[Balance],
		total_supply: Balance,
	) -> Option<(Balance, Vec<Balance>, Balance)> {
		let currencies_len = pool.currency_ids.len();
		let fee_pre_token = U256::from(Self::calculate_fee_per_token(pool)?);
		let amp = Self::get_a_precise(pool)?;

		let mut new_balances = pool.balances.clone();
		let d0 = U256::from(Self::get_d(&Self::xp(&pool.balances, &pool.token_multipliers)?, amp)?);

		for (i, x) in amounts.iter().enumerate() {
			new_balances[i] = new_balances[i].checked_sub(*x)?;
		}

		let d1 = U256::from(Self::get_d(&Self::xp(&new_balances, &pool.token_multipliers)?, amp)?);
		let mut fees = vec![Balance::default(); currencies_len];
		let fee_denominator = U256::from(FEE_DENOMINATOR);

		for (i, balance) in pool.balances.iter_mut().enumerate() {
			let ideal_balance = d1.checked_mul(U256::from(*balance))?.checked_div(d0)?;
			let diff = Self::distance(U256::from(new_balances[i]), ideal_balance);
			fees[i] = fee_pre_token
				.checked_mul(diff)?
				.checked_div(fee_denominator)
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())?;

			*balance = U256::from(new_balances[i])
				.checked_sub(
					U256::from(fees[i])
						.checked_mul(U256::from(pool.admin_fee))?
						.checked_div(fee_denominator)?,
				)
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())?;

			new_balances[i] = new_balances[i].checked_sub(fees[i])?;
		}

		let d1 = Self::get_d(&Self::xp(&new_balances, &pool.token_multipliers)?, amp)?;
		let burn_amount = d0
			.checked_sub(U256::from(d1))?
			.checked_mul(U256::from(total_supply))?
			.checked_div(d0)
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())?;

		Some((burn_amount, fees, d1))
	}

	pub fn get_a_precise(
		pool: &Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
	) -> Option<Number> {
		let now = T::TimeProvider::now().as_secs() as Number;

		if now >= pool.future_a_time {
			return Some(pool.future_a);
		}

		let future_a = U256::from(pool.future_a);
		let initial_a = U256::from(pool.initial_a);
		let now = U256::from(now);
		let future_a_time = U256::from(pool.future_a_time);
		let initial_a_time = U256::from(pool.initial_a_time);

		if pool.future_a > pool.initial_a {
			return initial_a
				.checked_add(
					future_a
						.checked_sub(initial_a)?
						.checked_mul(now.checked_sub(initial_a_time)?)?
						.checked_div(future_a_time.checked_sub(initial_a_time)?)?,
				)
				.and_then(|n| TryInto::<Balance>::try_into(n).ok());
		}

		initial_a
			.checked_sub(
				initial_a
					.checked_sub(future_a)?
					.checked_mul(now.checked_sub(initial_a_time)?)?
					.checked_div(future_a_time.checked_sub(initial_a_time)?)?,
			)
			.and_then(|n| TryInto::<Balance>::try_into(n).ok())
	}

	fn xp(balances: &[Balance], rates: &[Balance]) -> Option<Vec<Balance>> {
		let mut normalized_res = Vec::new();
		for (i, _) in balances.iter().enumerate() {
			normalized_res.push(balances[i].checked_mul(rates[i])?)
		}
		Some(normalized_res)
	}

	pub fn get_d(balances: &[Balance], amp: Balance) -> Option<Balance> {
		let n_currencies = Balance::from(balances.len() as u64);
		let sum = Self::sum_of(balances)?;
		if sum == Balance::default() {
			return Some(Balance::default());
		}
		let mut d_prev: U256;
		let mut d = U256::from(sum);
		let ann = U256::from(amp.checked_mul(n_currencies)?);
		let a_precision = U256::from(A_PRECISION);

		for _i in 0..MAX_ITERATION {
			let mut d_p = d;
			for b in balances.iter() {
				d_p = d_p
					.checked_mul(d)?
					.checked_div(U256::from(*b).checked_mul(U256::from(n_currencies))?)?;
			}
			d_prev = d;

			let numerator = ann
				.checked_mul(U256::from(sum))
				.and_then(|n| n.checked_div(a_precision))
				.and_then(|n| n.checked_add(d_p.checked_mul(U256::from(n_currencies))?))
				.and_then(|n| n.checked_mul(d))?;

			let denominator = ann
				.checked_sub(a_precision)
				.and_then(|n| n.checked_mul(d))
				.and_then(|n| n.checked_div(a_precision))
				.and_then(|n| {
					n.checked_add(
						U256::from(n_currencies)
							.checked_add(U256::from(1u32))?
							.checked_mul(d_p)?,
					)
				})?;

			d = numerator.checked_div(denominator)?;

			if Self::distance::<U256>(d, d_prev) <= U256::from(1u32) {
				return TryInto::<Balance>::try_into(d).ok();
			}
		}
		None
	}

	fn get_y(
		pool: &Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
		in_index: usize,
		out_index: usize,
		in_balance: Balance,
		normalize_balances: &[Balance],
	) -> Option<Balance> {
		let pool_currencies_len = pool.currency_ids.len();
		let n_currencies = U256::from(pool_currencies_len as u64);
		let amp = Self::get_a_precise(pool)?;
		let ann = n_currencies.checked_mul(U256::from(amp))?;
		let d = U256::from(Self::get_d(normalize_balances, amp)?);
		let mut c = d;
		let mut sum = U256::default();

		for (i, normalize_balance) in normalize_balances.iter().enumerate().take(pool_currencies_len) {
			if i == out_index {
				continue;
			}
			let x: Balance = if i == in_index { in_balance } else { *normalize_balance };

			sum = sum.checked_add(U256::from(x))?;

			c = c
				.checked_mul(d)?
				.checked_div(U256::from(x).checked_mul(n_currencies)?)?;
		}
		let a_percision = U256::from(A_PRECISION);
		c = c
			.checked_mul(d)?
			.checked_mul(a_percision)?
			.checked_div(ann.checked_mul(n_currencies)?)?;

		let b = sum.checked_add(d.checked_mul(a_percision)?.checked_div(ann)?)?;

		let mut last_y: U256;
		let mut y = d;
		for _i in 0..MAX_ITERATION {
			last_y = y;
			y = y
				.checked_mul(y)?
				.checked_add(c)?
				.checked_div(U256::from(2u32).checked_mul(y)?.checked_add(b)?.checked_sub(d)?)?;
			if Self::distance(last_y, y) <= U256::from(1) {
				return TryInto::<Balance>::try_into(y).ok();
			}
		}

		None
	}

	fn get_yd(
		pool: &Pool<T::CurrencyId, T::AccountId, BoundedVec<u8, T::PoolCurrencySymbolLimit>>,
		a: Balance,
		index: u32,
		xp: &[Balance],
		d: Balance,
	) -> Option<Balance> {
		let currencies_len = pool.currency_ids.len();
		if index >= currencies_len as u32 {
			return None;
		}

		let n_currencies = U256::from(currencies_len as u64);
		let ann = U256::from(a) * n_currencies;
		let mut c = U256::from(d);
		let mut s = U256::default();
		let _x: U256;
		let mut y_prev: U256;

		for (i, x) in xp.iter().enumerate() {
			if i as u32 == index {
				continue;
			}
			s = s.checked_add(U256::from(*x))?;
			c = c
				.checked_mul(U256::from(d))?
				.checked_div(U256::from(*x).checked_mul(n_currencies)?)?;
		}

		let a_precision = U256::from(A_PRECISION);
		c = c
			.checked_mul(U256::from(d))?
			.checked_mul(a_precision)?
			.checked_div(ann.checked_mul(n_currencies)?)?;
		let b = s.checked_add(U256::from(d).checked_mul(a_precision)?.checked_div(ann)?)?;
		let mut y = U256::from(d);

		for _i in 0..MAX_ITERATION {
			y_prev = y;
			y = y.checked_mul(y)?.checked_add(c)?.checked_div(
				U256::from(2u32)
					.checked_mul(y)?
					.checked_add(b)?
					.checked_sub(U256::from(d))?,
			)?;

			if Self::distance(y, y_prev) <= U256::from(1) {
				return TryInto::<Balance>::try_into(y).ok();
			}
		}

		None
	}

	fn sum_of(balances: &[Balance]) -> Option<Balance> {
		let mut sum = Balance::default();
		for b in balances.iter() {
			sum = sum.checked_add(*b)?
		}
		Some(sum)
	}

	fn do_transfer_in(
		currency_id: T::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Balance,
	) -> Result<Balance, Error<T>> {
		let to_prior_balance = T::MultiCurrency::free_balance(currency_id, to);
		T::MultiCurrency::transfer(currency_id, from, to, amount).map_err(|_| Error::<T>::InsufficientReserve)?;
		let to_new_balance = T::MultiCurrency::free_balance(currency_id, to);

		to_new_balance
			.checked_sub(to_prior_balance)
			.ok_or(Error::<T>::Arithmetic)
	}

	fn distance<Number: PartialOrd + Sub<Output = Number>>(x: Number, y: Number) -> Number {
		if x > y {
			x - y
		} else {
			y - x
		}
	}

	/// used for rpc
	pub fn calculate_currency_amount(
		pool_id: T::PoolId,
		amounts: Vec<Balance>,
		deposit: bool,
	) -> Result<Balance, DispatchError> {
		if let Some(pool) = Self::pools(pool_id) {
			ensure!(pool.currency_ids.len() == amounts.len(), Error::<T>::MismatchParameter);
			let amp = Self::get_a_precise(&pool).ok_or(Error::<T>::Arithmetic)?;

			let d0 = Self::xp(&pool.balances, &pool.token_multipliers)
				.and_then(|xp| Self::get_d(&xp, amp))
				.ok_or(Error::<T>::Arithmetic)?;

			let mut new_balances = pool.balances.clone();
			for (i, balance) in amounts.iter().enumerate() {
				if deposit {
					new_balances[i] = new_balances[i].checked_add(*balance).ok_or(Error::<T>::Arithmetic)?;
				} else {
					new_balances[i] = new_balances[i].checked_sub(*balance).ok_or(Error::<T>::Arithmetic)?;
				}
			}

			let d1 = Self::xp(&new_balances, &pool.token_multipliers)
				.and_then(|xp| Self::get_d(&xp, amp))
				.ok_or(Error::<T>::Arithmetic)?;

			let total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);

			if total_supply.is_zero() {
				return Ok(d1); // first depositor take it all
			}

			let diff: Balance = if deposit {
				d1.checked_sub(d0).ok_or(Error::<T>::Arithmetic)?
			} else {
				d0.checked_sub(d1).ok_or(Error::<T>::Arithmetic)?
			};

			let amount = U256::from(diff)
				.checked_mul(U256::from(total_supply))
				.and_then(|n| n.checked_div(U256::from(d0)))
				.and_then(|n| TryInto::<Balance>::try_into(n).ok())
				.ok_or(Error::<T>::Arithmetic)?;

			Ok(amount)
		} else {
			Err(Error::<T>::InvalidPoolId.into())
		}
	}

	pub fn calculate_virtual_price(pool_id: T::PoolId) -> Option<Balance> {
		if let Some(pool) = Self::pools(pool_id) {
			let d = Self::get_d(
				&Self::xp(&pool.balances, &pool.token_multipliers)?,
				Self::get_a_precise(&pool)?,
			)?;

			let total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);

			if total_supply > Zero::zero() {
				return U256::from(10)
					.checked_pow(U256::from(POOL_TOKEN_COMMON_DECIMALS))
					.and_then(|n| n.checked_mul(U256::from(d)))
					.and_then(|n| n.checked_div(U256::from(total_supply)))
					.and_then(|n| TryInto::<Balance>::try_into(n).ok());
			}
			None
		} else {
			None
		}
	}

	pub fn get_admin_balance(pool_id: T::PoolId, currency_index: usize) -> Option<Balance> {
		if let Some(pool) = Self::pools(pool_id) {
			let currencies_len = pool.currency_ids.len();
			if currency_index >= currencies_len {
				return None;
			}
			let balance = T::MultiCurrency::free_balance(pool.currency_ids[currency_index], &pool.account);

			balance.checked_sub(pool.balances[currency_index])
		} else {
			None
		}
	}
}
