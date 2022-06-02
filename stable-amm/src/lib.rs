// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

mod traits;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime, transactional, PalletId};
use sp_arithmetic::traits::{
	checked_pow, AtLeast32BitUnsigned, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Zero,
};
use sp_runtime::traits::{AccountIdConversion, StaticLookup};

use orml_traits::MultiCurrency;

pub use pallet::*;

#[allow(type_alias_bounds)]
type AccountIdOf<T: Config> = <T as frame_system::Config>::AccountId;

const FEE_DENOMINATOR: u32 = 1e10f64 as u32;
const A_PRECISION: u32 = 100;
const MAX_ITERATION: u32 = 255; // the number of iterations to sum d and y
pub const POOL_TOKEN_COMMON_DECIMALS: u8 = 18;

/// Some thresholds when setting the pool
const DAY: u32 = 24 * 60 * 60;
const MIN_RAMP_TIME: u32 = DAY;
const MAX_A: u32 = 1e6f32 as u32;
const MAX_A_CHANGE: u32 = 10u32;
pub const MAX_ADMIN_FEE: u32 = 1e10f64 as u32;
const MAX_SWAP_FEE: u32 = 1e8f64 as u32;

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct Pool<CurrencyId, Balance, AccountId> {
	pub pooled_currency_ids: Vec<CurrencyId>,
	pub lp_currency_id: CurrencyId,
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
	use frame_support::dispatch::{Codec, DispatchResult};
	use frame_system::pallet_prelude::*;
	use traits::ValidateCurrency;

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
			+ From<u64>
			+ From<u32>;

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
	#[pallet::getter(fn lp_currencies)]
	pub type LpCurrencies<T: Config> = StorageMap<_, Blake2_128Concat, T::CurrencyId, T::PoolId>;

	#[pallet::storage]
	#[pallet::getter(fn fee_receiver)]
	pub type FeeReceiver<T: Config> = StorageValue<_, Option<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreatePool(T::PoolId, Vec<T::CurrencyId>, T::CurrencyId, T::Balance, T::AccountId),
		AddLiquidity(
			T::PoolId,
			T::AccountId,
			Vec<T::Balance>,
			Vec<T::Balance>,
			T::Balance,
			T::Balance,
		),
		TokenExchange(T::PoolId, T::AccountId, u32, T::Balance, u32, T::Balance),
		RemoveLiquidity(T::PoolId, T::AccountId, Vec<T::Balance>, Vec<T::Balance>, T::Balance),
		RemoveLiquidityOneCurrency(T::PoolId, T::AccountId, u32, T::Balance, T::Balance),
		RemoveLiquidityImbalance(
			T::PoolId,
			T::AccountId,
			Vec<T::Balance>,
			Vec<T::Balance>,
			T::Balance,
			T::Balance,
		),
		NewFee(T::PoolId, T::Balance, T::Balance),
		RampA(T::PoolId, T::Balance, T::Balance, u64, u64),
		StopRampA(T::PoolId, T::Balance, u64),
		CollectProtocolFee(T::PoolId, T::CurrencyId, T::Balance),
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
		InsufficientSupply,
		InsufficientReserve,
		CheckDFailed,
		BalanceSlippage,
		SwapSameCurrency,
		CurrencyIndexOutRange,
		InsufficientLpReserve,
		TokenNotFound,
		ExceedThreshold,
		RampADelay,
		MinRampTime,
		ExceedMaxAChange,
		AlreadyStoppedRampA,
		NoFeeReceiver,
		MismatchParameter,
		ExceedMaxFee,
		ExceedMaxA,
		LpCurrencyAlreadyUsed,
		TXX,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			currency_ids: Vec<T::CurrencyId>,
			currency_decimals: Vec<u8>,
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
				Self::lp_currencies(lp_currency_id).is_none(),
				Error::<T>::LpCurrencyAlreadyUsed
			);

			ensure!(
				currency_ids.len() == currency_decimals.len(),
				Error::<T>::MismatchParameter
			);
			ensure!(a < T::Balance::from(MAX_A), Error::<T>::ExceedMaxA);
			ensure!(fee < T::Balance::from(MAX_SWAP_FEE), Error::<T>::ExceedMaxFee);
			ensure!(admin_fee < T::Balance::from(MAX_ADMIN_FEE), Error::<T>::ExceedMaxFee);

			let mut rate = Vec::new();

			for (i, _) in currency_ids.iter().enumerate() {
				ensure!(
					currency_decimals[i] <= POOL_TOKEN_COMMON_DECIMALS,
					Error::<T>::InvalidCurrencyDecimal
				);
				let r = checked_pow(
					T::Balance::from(10u32),
					(POOL_TOKEN_COMMON_DECIMALS - currency_decimals[i]) as usize,
				)
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

				LpCurrencies::<T>::insert(lp_currency_id, *next_pool_id);

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

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn swap(
			origin: OriginFor<T>,
			poo_id: T::PoolId,
			from_index: u32,
			to_index: u32,
			in_amount: T::Balance,
			min_out_amount: T::Balance,
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

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			poo_id: T::PoolId,
			lp_amount: T::Balance,
			min_amounts: Vec<T::Balance>,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			Self::inner_remove_liquidity(poo_id, &who, lp_amount, &min_amounts)?;

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn remove_liquidity_one_currency(
			origin: OriginFor<T>,
			poo_id: T::PoolId,
			lp_amount: T::Balance,
			index: u32,
			min_amount: T::Balance,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			Self::inner_remove_liquidity_one_currency(poo_id, &who, lp_amount, index, min_amount)?;

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn remove_liquidity_imbalance(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amounts: Vec<T::Balance>,
			max_burn_amount: T::Balance,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let now = frame_system::Pallet::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			Self::inner_remove_liquidity_imbalance(&who, pool_id, &amounts, max_burn_amount)?;

			Ok(())
		}

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
		pub fn update_fee(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			new_swap_fee: T::Balance,
			new_admin_fee: T::Balance,
		) -> DispatchResult {
			ensure_root(origin)?;
			Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
				let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
				ensure!(
					new_swap_fee <= T::Balance::from(MAX_SWAP_FEE),
					Error::<T>::ExceedThreshold
				);
				ensure!(
					new_admin_fee <= T::Balance::from(MAX_ADMIN_FEE),
					Error::<T>::ExceedThreshold
				);

				pool.admin_fee = new_admin_fee;
				pool.fee = new_swap_fee;

				Self::deposit_event(Event::NewFee(pool_id, new_swap_fee, new_admin_fee));
				Ok(())
			})
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn ramp_a(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			future_a: T::Balance,
			future_a_time: u64,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let timestamp = T::TimeProvider::now().as_secs();
			Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
				let now = T::Balance::try_from(timestamp).map_err(|_| Error::<T>::Arithmetic)?;

				let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;

				ensure!(
					now >= pool
						.initial_a_time
						.checked_add(&T::Balance::from(DAY))
						.ok_or(Error::<T>::Arithmetic)?,
					Error::<T>::RampADelay
				);

				ensure!(
					T::Balance::from(future_a_time)
						< now
							.checked_add(&T::Balance::from(MIN_RAMP_TIME))
							.ok_or(Error::<T>::Arithmetic)?,
					Error::<T>::MinRampTime
				);

				ensure!(
					future_a > Zero::zero() && future_a < T::Balance::from(MAX_A),
					Error::<T>::ExceedThreshold
				);

				let (initial_a_precise, future_a_precise) = Self::get_a_precise(pool)
					.and_then(|initial_a_precise| -> Option<(T::Balance, T::Balance)> {
						let future_a_precise = future_a.checked_mul(&T::Balance::from(A_PRECISION))?;
						Some((initial_a_precise, future_a_precise))
					})
					.ok_or(Error::<T>::Arithmetic)?;

				let max_a_change = T::Balance::from(MAX_A_CHANGE);

				if future_a_precise < initial_a_precise {
					ensure!(
						future_a_precise
							.checked_mul(&max_a_change)
							.ok_or(Error::<T>::Arithmetic)?
							>= initial_a_precise,
						Error::<T>::ExceedMaxAChange
					);
				} else {
					ensure!(
						future_a_precise
							<= initial_a_precise
								.checked_mul(&max_a_change)
								.ok_or(Error::<T>::Arithmetic)?,
						Error::<T>::ExceedMaxAChange
					);
				}

				pool.initial_a = initial_a_precise;
				pool.future_a = future_a_precise;
				pool.initial_a_time = now;
				pool.future_a_time = T::Balance::from(future_a_time);

				Self::deposit_event(Event::RampA(
					pool_id,
					initial_a_precise,
					future_a_precise,
					timestamp,
					future_a_time,
				));

				Ok(())
			})
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn stop_ramp_a(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_signed(origin)?;
			Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
				let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
				let timestamp = T::TimeProvider::now().as_secs();
				let now = T::Balance::try_from(timestamp).map_err(|_| Error::<T>::Arithmetic)?;
				ensure!(pool.future_a_time > now, Error::<T>::AlreadyStoppedRampA);

				let current_a = Self::get_a_precise(pool).ok_or(Error::<T>::Arithmetic)?;

				pool.initial_a = current_a;
				pool.future_a = current_a;
				pool.initial_a_time = now;
				pool.future_a_time = now;

				Self::deposit_event(Event::StopRampA(pool_id, current_a, timestamp));
				Ok(())
			})
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn withdraw_admin_fee(origin: OriginFor<T>, pool_id: T::PoolId) -> DispatchResult {
			ensure_root(origin)?;

			let receiver = Self::fee_receiver().ok_or(Error::<T>::NoFeeReceiver)?;

			Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
				let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
				for (i, reserve) in pool.balances.iter().enumerate() {
					let balance = T::MultiCurrency::free_balance(pool.pooled_currency_ids[i], &pool.pool_account)
						.checked_sub(reserve)
						.ok_or(Error::<T>::Arithmetic)?;

					if !balance.is_zero() {
						T::MultiCurrency::transfer(
							pool.pooled_currency_ids[i],
							&pool.pool_account,
							&receiver,
							balance,
						)?;
					}
					Self::deposit_event(Event::CollectProtocolFee(pool_id, pool.pooled_currency_ids[i], balance));
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
		amounts: &[T::Balance],
		min_mint_amount: T::Balance,
	) -> DispatchResult {
		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;

			let n_currencies = pool.pooled_currency_ids.len();
			ensure!(n_currencies == amounts.len(), Error::<T>::InvalidParameter);
			let mut fees = Vec::new();
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

			let mut new_balances = pool.balances.clone();

			for i in 0..n_currencies {
				if lp_total_supply == Zero::zero() {
					ensure!(!amounts[i].is_zero(), Error::<T>::InsufficientSupply);
				}
				new_balances[i] = new_balances[i]
					.checked_add(&Self::do_transfer_in(
						pool.pooled_currency_ids[i],
						&who,
						&pool.pool_account,
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

			let mint_amount: T::Balance;
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

			ensure!(min_mint_amount <= mint_amount, Error::<T>::BalanceSlippage);

			T::MultiCurrency::deposit(pool.lp_currency_id, &who, mint_amount)?;

			Self::deposit_event(Event::AddLiquidity(
				pool_id,
				who.clone(),
				amounts.to_vec(),
				fees.to_vec(),
				d1,
				mint_amount,
			));
			Ok(())
		})
	}

	pub fn inner_swap(
		who: &T::AccountId,
		pool_id: T::PoolId,
		i: usize,
		j: usize,
		in_amount: T::Balance,
		out_min_amount: T::Balance,
	) -> DispatchResult {
		ensure!(i != j, Error::<T>::SwapSameCurrency);

		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
			let n_currencies = pool.pooled_currency_ids.len();
			ensure!(i < n_currencies && j < n_currencies, Error::<T>::CurrencyIndexOutRange);

			let in_amount = Self::do_transfer_in(pool.pooled_currency_ids[i], who, &pool.pool_account, in_amount)?;

			let (dy, admin_fee) = Self::calculate_swap_amount(pool, i, j, in_amount).ok_or(Error::<T>::Arithmetic)?;
			ensure!(dy >= out_min_amount, Error::<T>::BalanceSlippage);

			pool.balances[i] = pool.balances[i].checked_add(&in_amount).ok_or(Error::<T>::Arithmetic)?;
			pool.balances[j] = pool.balances[j]
				.checked_sub(&dy.checked_add(&admin_fee).ok_or(Error::<T>::Arithmetic)?)
				.ok_or(Error::<T>::Arithmetic)?;

			T::MultiCurrency::transfer(pool.pooled_currency_ids[j], &pool.pool_account, &who, dy)?;

			Self::deposit_event(Event::TokenExchange(
				pool_id,
				who.clone(),
				i as u32,
				in_amount,
				j as u32,
				dy,
			));
			Ok(())
		})
	}

	pub fn inner_remove_liquidity(
		pool_id: T::PoolId,
		who: &T::AccountId,
		lp_amount: T::Balance,
		min_amounts: &[T::Balance],
	) -> DispatchResult {
		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
			let lp_total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);

			ensure!(lp_total_supply >= lp_amount, Error::<T>::InsufficientReserve);

			let currencies_length = pool.pooled_currency_ids.len();
			let fees: Vec<T::Balance> = vec![Zero::zero(); currencies_length];
			let amounts = Self::calculate_removed_liquidity(pool, lp_amount).ok_or(Error::<T>::Arithmetic)?;

			for (i, amount) in amounts.iter().enumerate() {
				ensure!(*amount >= min_amounts[i], Error::<T>::BalanceSlippage);
				pool.balances[i] = pool.balances[i].checked_sub(&amount).ok_or(Error::<T>::Arithmetic)?;
				T::MultiCurrency::transfer(pool.pooled_currency_ids[i], &pool.pool_account, &who, *amount)?;
			}

			T::MultiCurrency::withdraw(pool.lp_currency_id, &who, lp_amount)?;
			Self::deposit_event(Event::RemoveLiquidity(
				pool_id,
				who.clone(),
				amounts,
				fees,
				lp_total_supply - lp_amount,
			));
			Ok(())
		})
	}

	pub fn inner_remove_liquidity_one_currency(
		pool_id: T::PoolId,
		who: &T::AccountId,
		lp_amount: T::Balance,
		index: u32,
		min_amount: T::Balance,
	) -> DispatchResult {
		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
			let total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);
			ensure!(total_supply > Zero::zero(), Error::<T>::InsufficientLpReserve);
			ensure!(
				T::MultiCurrency::free_balance(pool.lp_currency_id, who) >= lp_amount && lp_amount <= total_supply,
				Error::<T>::InsufficientSupply
			);
			ensure!(index < pool.pooled_currency_ids.len() as u32, Error::<T>::TokenNotFound);

			let (dy, dy_fee) = Self::calculate_remove_liquidity_one_token(pool, lp_amount, index, total_supply)
				.ok_or(Error::<T>::Arithmetic)?;

			ensure!(dy >= min_amount, Error::<T>::BalanceSlippage);
			let fee_denominator = T::Balance::from(FEE_DENOMINATOR);

			pool.balances[index as usize] = dy_fee
				.checked_mul(&pool.admin_fee)
				.and_then(|n| n.checked_div(&fee_denominator))
				.and_then(|admin_fee| pool.balances[index as usize].checked_sub(&admin_fee))
				.ok_or(Error::<T>::Arithmetic)?;

			T::MultiCurrency::withdraw(pool.lp_currency_id, &who, lp_amount)?;

			T::MultiCurrency::transfer(pool.lp_currency_id, &pool.pool_account, &who, dy)?;

			Self::deposit_event(Event::RemoveLiquidityOneCurrency(
				pool_id,
				who.clone(),
				index,
				lp_amount,
				dy,
			));
			Ok(())
		})
	}

	pub fn inner_remove_liquidity_imbalance(
		who: &T::AccountId,
		pool_id: T::PoolId,
		amounts: &[T::Balance],
		max_burn_amount: T::Balance,
	) -> DispatchResult {
		Pools::<T>::try_mutate_exists(pool_id, |optioned_pool| -> DispatchResult {
			let pool = optioned_pool.as_mut().ok_or(Error::<T>::InvalidPoolId)?;
			let total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);

			ensure!(total_supply > Zero::zero(), Error::<T>::InsufficientLpReserve);
			ensure!(
				amounts.len() == pool.pooled_currency_ids.len(),
				Error::<T>::InvalidParameter
			);

			let (burn_amount, fees, d1) = Self::calculate_remove_liquidity_imbalance(pool, amounts, total_supply)
				.ok_or(Error::<T>::Arithmetic)?;
			ensure!(
				burn_amount > Zero::zero() && burn_amount <= max_burn_amount,
				Error::<T>::BalanceSlippage
			);

			T::MultiCurrency::withdraw(pool.lp_currency_id, &who, burn_amount)?;

			for (i, balance) in amounts.iter().enumerate() {
				if *balance > Zero::zero() {
					T::MultiCurrency::transfer(pool.pooled_currency_ids[i], &pool.pool_account, &who, *balance)?;
				}
			}

			Self::deposit_event(Event::RemoveLiquidityImbalance(
				pool_id,
				who.clone(),
				amounts.to_vec(),
				fees,
				d1,
				total_supply - burn_amount,
			));

			Ok(())
		})
	}

	fn calculate_fee_per_token(pool: &Pool<T::CurrencyId, T::Balance, T::AccountId>) -> Option<T::Balance> {
		let n_pooled_currency = T::Balance::from(pool.pooled_currency_ids.len() as u64);

		pool.fee
			.checked_mul(&n_pooled_currency)?
			.checked_div(&T::Balance::from(4u32).checked_mul(&n_pooled_currency.checked_sub(&One::one())?)?)
	}

	fn calculate_mint_amount(
		pool: &mut Pool<T::CurrencyId, T::Balance, T::AccountId>,
		new_balances: &mut [T::Balance],
		d0: T::Balance,
		d1: &mut T::Balance,
		fee: T::Balance,
		amp: T::Balance,
		total_supply: T::Balance,
	) -> Option<(T::Balance, Vec<T::Balance>)> {
		let mut diff: T::Balance;
		let n_currencies = pool.pooled_currency_ids.len();
		let fee_denominator = T::Balance::from(FEE_DENOMINATOR);
		let mut fees = vec![Zero::zero(); n_currencies];

		for i in 0..n_currencies {
			diff = Self::distance(d1.checked_mul(&pool.balances[i])?.checked_div(&d0)?, new_balances[i]);
			fees[i] = fee.checked_mul(&diff)?.checked_div(&fee_denominator)?;
			pool.balances[i] =
				new_balances[i].checked_sub(&fees[i].checked_mul(&pool.admin_fee)?.checked_div(&fee_denominator)?)?;
			new_balances[i] = new_balances[i].checked_sub(&fees[i])?;
		}
		*d1 = Self::get_d(&Self::xp(new_balances, &pool.token_multipliers)?, amp)?;

		let mint_amount = total_supply.checked_mul(&d1.checked_sub(&d0)?)?.checked_div(&d0)?;

		Some((mint_amount, fees))
	}

	fn calculate_swap_amount(
		pool: &mut Pool<T::CurrencyId, T::Balance, T::AccountId>,
		i: usize,
		j: usize,
		in_balance: T::Balance,
	) -> Option<(T::Balance, T::Balance)> {
		let fee_denominator = T::Balance::from(FEE_DENOMINATOR);

		let normalize_balances = Self::xp(&pool.balances, &pool.token_multipliers)?;
		let x = normalize_balances[i].checked_add(&in_balance.checked_mul(&pool.token_multipliers[i])?)?;
		let y = Self::get_y(pool, i, j, x, &normalize_balances)?;

		let mut dy = normalize_balances[j].checked_sub(&y)?.checked_sub(&One::one())?;
		let dy_fee = dy.checked_mul(&pool.fee)?.checked_div(&fee_denominator)?;
		dy = dy.checked_sub(&dy_fee)?.checked_div(&pool.token_multipliers[i])?;

		let admin_fee = dy_fee
			.checked_mul(&pool.admin_fee)?
			.checked_div(&fee_denominator)?
			.checked_div(&pool.token_multipliers[j])?;

		Some((dy, admin_fee))
	}

	fn calculate_removed_liquidity(
		pool: &mut Pool<T::CurrencyId, T::Balance, T::AccountId>,
		amount: T::Balance,
	) -> Option<Vec<T::Balance>> {
		let lp_total_supply = T::MultiCurrency::total_issuance(pool.lp_currency_id);
		if lp_total_supply < amount {
			return None;
		}
		let mut amounts = Vec::new();
		for b in pool.balances.iter() {
			amounts.push(b.checked_mul(&amount)?.checked_div(&lp_total_supply)?);
		}
		Some(amounts)
	}

	fn calculate_remove_liquidity_one_token(
		pool: &mut Pool<T::CurrencyId, T::Balance, T::AccountId>,
		token_amount: T::Balance,
		index: u32,
		total_supply: T::Balance,
	) -> Option<(T::Balance, T::Balance)> {
		if index < pool.pooled_currency_ids.len() as u32 {
			return None;
		}
		let amp = Self::get_a_precise(pool)?;
		let xp = Self::xp(&pool.balances, &pool.token_multipliers)?;
		let d0 = Self::get_d(&pool.balances, amp)?;
		let d1 = d0.checked_sub(&token_amount.checked_mul(&d0)?.checked_div(&total_supply)?)?;
		let new_y = Self::get_yd(pool, amp, index, &xp, d1)?;
		let mut reduce_xp = xp.clone();
		let fee_pre_token = Self::calculate_fee_per_token(pool)?;
		let fee_denominator = T::Balance::from(FEE_DENOMINATOR);

		for (i, x) in xp.iter().enumerate() {
			let expected_dx: T::Balance;
			if i as u32 == index {
				expected_dx = x.checked_mul(&d1)?.checked_div(&d0)?.checked_sub(&new_y)?;
			} else {
				expected_dx = x.checked_sub(&x.checked_mul(&d1)?.checked_div(&d0)?)?;
			}
			reduce_xp[i] =
				reduce_xp[i].checked_sub(&fee_pre_token.checked_mul(&expected_dx)?.checked_div(&fee_denominator)?)?;
		}
		let mut dy = reduce_xp[index as usize].checked_sub(&Self::get_yd(pool, amp, index, &reduce_xp, d1)?)?;
		dy = dy
			.checked_sub(&One::one())?
			.checked_div(&pool.token_multipliers[index as usize])?;

		let fee = xp[index as usize]
			.checked_sub(&new_y)?
			.checked_div(&pool.token_multipliers[index as usize])?
			.checked_sub(&dy)?;

		Some((dy, fee))
	}

	fn calculate_remove_liquidity_imbalance(
		pool: &mut Pool<T::CurrencyId, T::Balance, T::AccountId>,
		amounts: &[T::Balance],
		total_supply: T::Balance,
	) -> Option<(T::Balance, Vec<T::Balance>, T::Balance)> {
		let currencies_len = pool.pooled_currency_ids.len();
		let fee_pre_token = Self::calculate_fee_per_token(pool)?;
		let amp = Self::get_a_precise(pool)?;

		let mut new_balances = pool.balances.clone();
		let d0 = Self::get_d(&Self::xp(&pool.balances, &pool.token_multipliers)?, amp)?;

		for (i, x) in amounts.iter().enumerate() {
			new_balances[i] = new_balances[i].checked_sub(x)?;
		}

		let d1 = Self::get_d(&Self::xp(&new_balances, &pool.token_multipliers)?, amp)?;
		let mut fees = vec![T::Balance::default(); currencies_len];
		let fee_denominator = T::Balance::from(FEE_DENOMINATOR);

		for (i, balance) in pool.balances.iter_mut().enumerate() {
			let ideal_balance = d1.checked_mul(balance)?.checked_div(&d0)?;
			let diff = Self::distance(new_balances[i], ideal_balance);
			fees[i] = fee_pre_token.checked_mul(&diff)?.checked_div(&fee_denominator)?;
			*balance =
				new_balances[i].checked_sub(&fees[i].checked_mul(&pool.admin_fee)?.checked_div(&fee_denominator)?)?;
			new_balances[i] = new_balances[i].checked_sub(&fees[i])?;
		}

		let d1 = Self::get_d(&Self::xp(&new_balances, &pool.token_multipliers)?, amp)?;
		let burn_amount = d0.checked_sub(&d1)?.checked_mul(&total_supply)?;

		Some((burn_amount, fees, d1))
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
		let n_currencies = T::Balance::from(balances.len() as u64);
		let sum = Self::sum_of(balances)?;
		if sum == T::Balance::default() {
			return Some(T::Balance::default());
		}

		let mut d_prev: T::Balance;
		let mut d = sum;
		let ann = amp.checked_div(&n_currencies)?;
		let a_precision = T::Balance::from(A_PRECISION);

		for _i in 0..MAX_ITERATION {
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

			if Self::distance(d, d_prev) <= One::one() {
				return Some(d);
			}
		}
		None
	}

	fn get_y(
		pool: &Pool<T::CurrencyId, T::Balance, T::AccountId>,
		in_index: usize,
		out_index: usize,
		in_balance: T::Balance,
		normalize_balances: &[T::Balance],
	) -> Option<T::Balance> {
		let pool_currencies_len = pool.pooled_currency_ids.len();
		let n_currencies = T::Balance::from(pool_currencies_len as u64);
		let amp = Self::get_a_precise(pool)?;
		let ann = T::Balance::from(n_currencies).checked_mul(&amp)?;
		let d = Self::get_d(normalize_balances, amp)?;
		let mut c = d;
		let mut sum = T::Balance::default();

		for i in 0..pool_currencies_len {
			if i == out_index {
				continue;
			}
			let x: T::Balance;
			if i == in_index {
				x = in_balance;
			} else {
				x = normalize_balances[i];
			}
			sum = sum.checked_add(&x)?;

			c = c.checked_mul(&d)?.checked_div(&ann.checked_div(&n_currencies)?)?;
		}

		let a_percision = T::Balance::from(A_PRECISION);
		c = c
			.checked_mul(&d)?
			.checked_mul(&a_percision)?
			.checked_div(&ann.checked_mul(&n_currencies)?)?;

		let b = sum.checked_add(&d.checked_mul(&a_percision)?)?.checked_div(&ann)?;

		let mut last_y: T::Balance;
		let mut y = d;
		for _i in 0..MAX_ITERATION {
			last_y = y;
			y = y.checked_mul(&y)?.checked_add(&c)?.checked_div(
				&T::Balance::from(2u32)
					.checked_mul(&y)?
					.checked_add(&b)?
					.checked_sub(&d)?,
			)?;
			if Self::distance(last_y, y) <= One::one() {
				return Some(y);
			}
		}

		None
	}

	fn get_yd(
		pool: &Pool<T::CurrencyId, T::Balance, T::AccountId>,
		a: T::Balance,
		index: u32,
		xp: &[T::Balance],
		d: T::Balance,
	) -> Option<T::Balance> {
		let currencies_len = pool.pooled_currency_ids.len();
		if currencies_len as u32 <= index {
			return None;
		}
		let n_currencies = T::Balance::from(currencies_len as u64);
		let ann = a * n_currencies;
		let mut c = d;
		let mut s = T::Balance::default();
		let _x: T::Balance;
		let mut y_prev: T::Balance;

		for (i, x) in xp.iter().enumerate() {
			if i as u32 == index {
				continue;
			}
			s = s.checked_add(&x)?;
			c = c.checked_mul(&d)?.checked_div(&x.checked_mul(&n_currencies)?)?;
		}
		let a_precision = T::Balance::from(A_PRECISION);
		c = c
			.checked_mul(&d)?
			.checked_mul(&a_precision)?
			.checked_div(&ann.checked_mul(&n_currencies)?)?;
		let b = s.checked_add(&d.checked_mul(&a_precision)?.checked_div(&ann)?)?;
		let mut y = d;

		for _i in 0..MAX_ITERATION {
			y_prev = y;
			y = y.checked_mul(&y)?.checked_add(&c)?.checked_div(
				&T::Balance::from(2u32)
					.checked_mul(&y)?
					.checked_add(&b)?
					.checked_sub(&d)?,
			)?;

			if Self::distance(y, y_prev) <= One::one() {
				return Some(y);
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

	fn do_transfer_in(
		currency_id: T::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: T::Balance,
	) -> Result<T::Balance, Error<T>> {
		let to_prior_balance = T::MultiCurrency::free_balance(currency_id, to);
		T::MultiCurrency::transfer(currency_id, from, to, amount).map_err(|_| Error::<T>::InsufficientReserve)?;
		let to_new_balance = T::MultiCurrency::free_balance(currency_id, to);

		to_new_balance
			.checked_sub(&to_prior_balance)
			.ok_or(Error::<T>::Arithmetic)
	}

	fn distance(x: T::Balance, y: T::Balance) -> T::Balance {
		if x > y {
			x - y
		} else {
			y - x
		}
	}
}
