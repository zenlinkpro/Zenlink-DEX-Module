// Copyright 2021-2022 Zenlink
// Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

pub mod primitives;
pub use pallet::*;
use primitives::*;

use codec::{Codec, Decode, Encode};
use sp_arithmetic::traits::{One, Zero};
use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, StaticLookup};

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::UnixTime, PalletId};

use orml_traits::MultiCurrency;

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

		/// The currency ID type
		type CurrencyId: Parameter
			+ Member
			+ Copy
			+ MaybeSerializeDeserialize
			+ Ord
			+ TypeInfo
			+ MaxEncodedLen;

		/// The trait control all currencies
		type MultiCurrency: MultiCurrency<
			AccountIdOf<Self>,
			CurrencyId = Self::CurrencyId,
			Balance = Balance,
		>;

		/// The pool ID type
		type PoolId: Parameter + Codec + Copy + Ord + AtLeast32BitUnsigned + Zero + One + Default;

		/// The trait get timestamp of chain.
		type TimeProvider: UnixTime;

		/// This pallet id.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// The next vote period id.
	#[pallet::storage]
	#[pallet::getter(fn next_period_id)]
	pub(crate) type NextPeriodId<T: Config> = StorageValue<_, PeriodId, ValueQuery>;

	/// The duration between tow vote period.
	#[pallet::storage]
	#[pallet::getter(fn vote_duration)]
	pub(super) type VoteDuration<T: Config> = StorageValue<_, Duration, ValueQuery>;

	/// The duration of a vote period.
	#[pallet::storage]
	#[pallet::getter(fn vote_set_window)]
	pub(super) type VoteSetWindow<T: Config> = StorageValue<_, Duration, ValueQuery>;

	/// The Currency of the token used to vote.
	#[pallet::storage]
	#[pallet::getter(fn vote_currency)]
	pub(super) type VoteCurrency<T: Config> = StorageValue<_, Option<T::CurrencyId>, ValueQuery>;

	/// (periodId => VotePeriod)
	#[pallet::storage]
	#[pallet::getter(fn vote_period)]
	pub type VotePeriods<T: Config> = StorageMap<_, Twox64Concat, PeriodId, VotePeriod>;

	/// The amounts of a specific account for a specific pool.
	#[pallet::storage]
	#[pallet::getter(fn account_vote_amount)]
	pub type AccountVoteAmount<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, T::PoolId, Balance>;

	/// The state of a pool in all period.
	#[pallet::storage]
	#[pallet::getter(fn global_pool_state)]
	pub type GlobalPoolState<T: Config> =
		StorageDoubleMap<_, Twox64Concat, PeriodId, Twox64Concat, T::PoolId, PoolState>;

	/// The period which the Pool last changed state.
	#[pallet::storage]
	#[pallet::getter(fn pool_last_update_period)]
	pub type PoolLastUpdatePerold<T: Config> = StorageMap<_, Twox64Concat, T::PoolId, PeriodId>;

	#[pallet::storage]
	#[pallet::getter(fn admin)]
	pub type Admin<T: Config> = StorageValue<_, Option<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		UpdateAdmin {
			new_admin: T::AccountId,
		},
		UpdateVoteSetWindow {
			current_period: PeriodId,
			vote_set_window: Duration,
		},
		UpdateVoteDuration {
			current_period: PeriodId,
			vote_duration: Duration,
		},
		UpdateVotePeriod {
			period: PeriodId,
			start: Timestamp,
			end: Timestamp,
		},
		SetVotablePools {
			period: PeriodId,
			pools: Vec<T::PoolId>,
		},
		SetNonVotablePools {
			period: PeriodId,
			pools: Vec<T::PoolId>,
		},
		InheritPool {
			pool_id: T::PoolId,
			current_period_id: PeriodId,
			last_update_period_id: PeriodId,
			amount: Balance,
			votable: bool,
		},
		Vote {
			caller: T::AccountId,
			period_id: PeriodId,
			pool_id: T::PoolId,
			amount: Balance,
		},
		CancelVote {
			caller: T::AccountId,
			period_id: PeriodId,
			pool_id: T::PoolId,
			amount: Balance,
		},
		UpdateHistory {
			pool_id: T::PoolId,
			current_period_id: PeriodId,
			last_period: PeriodId,
			update_period: PeriodId,
			last_period_amount: Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The gauge pallet already been initialized.
		Initialized,
		Math,
		OnlyAdmin,
		InvalidTimestamp,
		Uninitialized,
		UnexpiredPeriod,
		InvalidPeriodId,
		InvalidPoolId,
		InvalidPoolState,
		ExpiredPeriod,
		NonVotablePool,
		InsufficientAmount,
		MismatchParameters,
		NoNeedUpdate,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10000)]
		#[transactional]
		pub fn update_admin(
			origin: OriginFor<T>,
			admin: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			ensure_root(origin)?;
			let new_admin = T::Lookup::lookup(admin)?;

			Admin::<T>::mutate(|old_admin| *old_admin = Some(new_admin.clone()));

			Self::deposit_event(Event::UpdateAdmin { new_admin });

			Ok(())
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn initialize(
			origin: OriginFor<T>,
			vote_currency: T::CurrencyId,
			vote_duration: Duration,
			vote_set_window: Duration,
			start: Timestamp,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::admin() == Some(who), Error::<T>::OnlyAdmin);

			ensure!(T::TimeProvider::now().as_secs() < start, Error::<T>::InvalidTimestamp);

			NextPeriodId::<T>::try_mutate(|current_period_id| -> DispatchResult {
				ensure!(*current_period_id == 0u32, Error::<T>::Initialized);

				VoteDuration::<T>::mutate(|duration| *duration = vote_duration);
				VoteSetWindow::<T>::mutate(|window| *window = vote_set_window);
				VoteCurrency::<T>::mutate(|currency| *currency = Some(vote_currency));

				VotePeriods::<T>::insert(
					*current_period_id,
					VotePeriod {
						start,
						end: start.checked_add(vote_duration).ok_or(Error::<T>::Math)?,
					},
				);

				*current_period_id = One::one();

				Ok(())
			})
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn update_vote_set_window(
			origin: OriginFor<T>,
			vote_set_window: Duration,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::admin() == Some(who), Error::<T>::OnlyAdmin);
			let next_period_id = Self::next_period_id();
			ensure!(next_period_id != 0u32, Error::<T>::Uninitialized);

			VoteSetWindow::<T>::mutate(|window| *window = vote_set_window);

			Self::deposit_event(Event::UpdateVoteSetWindow {
				current_period: next_period_id - 1,
				vote_set_window,
			});
			Ok(())
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn update_vote_duration(
			origin: OriginFor<T>,
			vote_duration: Duration,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::admin() == Some(who), Error::<T>::OnlyAdmin);
			let next_period_id = Self::next_period_id();
			ensure!(next_period_id != 0u32, Error::<T>::Uninitialized);

			VoteDuration::<T>::mutate(|duration| *duration = vote_duration);

			Self::deposit_event(Event::UpdateVoteDuration {
				current_period: next_period_id - 1,
				vote_duration,
			});

			Ok(())
		}

		/// Try to start a new period. If the current period has not expired, then it will fail.
		#[pallet::weight(10000)]
		#[transactional]
		pub fn update_vote_period(origin: OriginFor<T>) -> DispatchResult {
			ensure_signed(origin)?;
			let now = T::TimeProvider::now().as_secs();
			Self::inner_update_vote_periold(now)
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn set_voteable_pools(origin: OriginFor<T>, pools: Vec<T::PoolId>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::admin() == Some(who), Error::<T>::OnlyAdmin);

			let mut current_period_id =
				Self::get_current_period().ok_or(Error::<T>::InvalidPeriodId)?;
			let current_period =
				Self::vote_period(current_period_id).ok_or(Error::<T>::InvalidPeriodId)?;

			let now = T::TimeProvider::now().as_secs();

			// if current period expired, then update the pool state in next period.
			if now > current_period.end {
				current_period_id =
					current_period_id.checked_add(One::one()).ok_or(Error::<T>::Math)?;
			}

			Self::update_votable(true, current_period_id, &pools)?;

			Self::deposit_event(Event::SetVotablePools { period: current_period_id, pools });
			Ok(())
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn set_non_voteable_pools(
			origin: OriginFor<T>,
			pools: Vec<T::PoolId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::admin() == Some(who), Error::<T>::OnlyAdmin);

			let mut current_period_id =
				Self::get_current_period().ok_or(Error::<T>::InvalidPeriodId)?;
			let current_period =
				Self::vote_period(current_period_id).ok_or(Error::<T>::InvalidPeriodId)?;

			let now = T::TimeProvider::now().as_secs();

			// if current period expired, then update the pool state in next period.
			if now > current_period.end {
				current_period_id =
					current_period_id.checked_add(One::one()).ok_or(Error::<T>::Math)?;
			}

			Self::update_votable(false, current_period_id, &pools)?;

			Self::deposit_event(Event::SetNonVotablePools { period: current_period_id, pools });
			Ok(())
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn vote(origin: OriginFor<T>, pool_id: T::PoolId, amounts: Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = T::TimeProvider::now().as_secs();

			Self::inner_update_vote_periold(now)?;

			let current_period_id = Self::get_current_period().ok_or(Error::<T>::Math)?;

			let vote_period =
				Self::vote_period(current_period_id).ok_or(Error::<T>::InvalidPeriodId)?;
			ensure!(now < vote_period.end, Error::<T>::ExpiredPeriod);

			GlobalPoolState::<T>::try_mutate(
				current_period_id,
				pool_id,
				|pool_state| -> DispatchResult {
					let mut new_state = PoolState::default();
					if let Some(state) = pool_state {
						new_state = *state;
					}

					Self::inherit_expired_pool(pool_id, &mut new_state, current_period_id)?;
					Self::inner_vote(
						&mut new_state,
						&who,
						amounts,
						pool_id,
						current_period_id,
						&vote_period,
						now,
					)?;

					*pool_state = Some(new_state);
					Ok(())
				},
			)
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn cancel_vote(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amounts: Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let now = T::TimeProvider::now().as_secs();
			Self::inner_update_vote_periold(now)?;

			let current_period_id = Self::get_current_period().ok_or(Error::<T>::Math)?;

			GlobalPoolState::<T>::try_mutate(
				current_period_id,
				pool_id,
				|pool_state| -> DispatchResult {
					// if let Some(state) = pool_state {

					// 	Self::inherit_expired_pool(pool_id, state, current_period_id)?;

					// 	Self::inner_cancel_vote(
					// 		state,
					// 		&who,
					// 		amounts,
					// 		pool_id,
					// 		current_period_id,
					// 		now,
					// 	)
					// } else {
					// 	Err(Error::<T>::InvalidPeriodId.into())
					// }
					let mut new_state = PoolState::default();
					if let Some(state) = pool_state {
						new_state = *state;
					}
					Self::inherit_expired_pool(pool_id, &mut new_state, current_period_id)?;
					Self::inner_cancel_vote(
						&mut new_state,
						&who,
						amounts,
						pool_id,
						current_period_id,
						now,
					)?;

					*pool_state = Some(new_state);
					Ok(())
				},
			)
		}

		#[pallet::weight(10000)]
		#[transactional]
		pub fn update_pool_histroy(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			need_update_period_id: PeriodId,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let current_period_id = Self::get_current_period().ok_or(Error::<T>::Math)?;
			ensure!(
				need_update_period_id != 0 && current_period_id >= need_update_period_id,
				Error::<T>::NoNeedUpdate
			);

			for period_id in (0..need_update_period_id - 1).rev() {
				if let Some(last_update_pool_state) = Self::global_pool_state(period_id, pool_id) {
					if last_update_pool_state.inherit || period_id == 0 {
						for try_update_period_id in period_id + 1..=need_update_period_id {
							GlobalPoolState::<T>::try_mutate(
								try_update_period_id,
								pool_id,
								|update_state| -> DispatchResult {
									if let Some(state) = update_state {
										if state.inherit {
											return Ok(())
										}
										state.inherit = true;
										state.score = last_update_pool_state.total_amount;
										state.total_amount = last_update_pool_state.total_amount;
										if !state.reset_votable {
											state.votable = last_update_pool_state.votable;
										}
									} else {
										*update_state = Some(PoolState {
											inherit: true,
											reset_votable: false,
											votable: last_update_pool_state.votable,
											score: last_update_pool_state.total_amount,
											total_amount: last_update_pool_state.total_amount,
										})
									}
									Ok(())
								},
							)?;
						}
						Self::deposit_event(Event::UpdateHistory {
							pool_id,
							current_period_id,
							last_period: period_id,
							update_period: need_update_period_id,
							last_period_amount: last_update_pool_state.total_amount,
						});
					}
				}
			}
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_current_period() -> Option<PeriodId> {
		Self::next_period_id().checked_sub(One::one())
	}

	fn update_votable(votable: bool, period: PeriodId, pools: &[T::PoolId]) -> DispatchResult {
		for pool_id in pools {
			GlobalPoolState::<T>::try_mutate(period, pool_id, |pool_state| -> DispatchResult {
				if let Some(state) = pool_state {
					state.votable = votable;
					state.reset_votable = true;
				} else {
					*pool_state =
						Some(PoolState { votable, reset_votable: true, ..Default::default() });
				}
				Ok(())
			})?;
		}
		Ok(())
	}

	fn inner_update_vote_periold(now: Timestamp) -> DispatchResult {
		NextPeriodId::<T>::try_mutate(|next_period_id| -> DispatchResult {
			let current_period_id =
				next_period_id.checked_sub(One::one()).ok_or(Error::<T>::Math)?;
			let current_period =
				Self::vote_period(current_period_id).ok_or(Error::<T>::InvalidPeriodId)?;

			if current_period.end > now {
				return Ok(())
			}
			VotePeriods::<T>::try_mutate(*next_period_id, |period| -> DispatchResult {
				let vote_set_window = Self::vote_set_window();
				let vote_duration = Self::vote_duration();

				let mut next_period = VotePeriod::default();
				let next_period_start =
					current_period.end.checked_add(vote_set_window).ok_or(Error::<T>::Math)?;
				if next_period_start >= now {
					next_period.start = next_period_start;
				} else {
					next_period.start = now;
				}
				next_period.end =
					next_period.start.checked_add(vote_duration).ok_or(Error::<T>::Math)?;

				Self::deposit_event(Event::UpdateVotePeriod {
					period: *next_period_id,
					start: next_period.start,
					end: next_period.end,
				});

				*period = Some(next_period);

				Ok(())
			})?;

			*next_period_id = next_period_id.checked_add(One::one()).ok_or(Error::<T>::Math)?;

			Ok(())
		})
	}

	fn inner_vote(
		pool_state: &mut PoolState,
		who: &T::AccountId,
		amount: Balance,
		pool_id: T::PoolId,
		current_period_id: PeriodId,
		current_perold: &VotePeriod,
		mut now: Timestamp,
	) -> DispatchResult {
		if !pool_state.votable {
			return Err(Error::<T>::NonVotablePool.into())
		}
		let vote_currency = Self::vote_currency().ok_or(Error::<T>::Uninitialized)?;
		let pallet_account = T::PalletId::get().into_account_truncating();
		T::MultiCurrency::transfer(vote_currency, who, &pallet_account, amount)?;

		if now < current_perold.start {
			now = current_perold.start;
		}

		let calculate_result = current_perold
			.end
			.checked_sub(now)
			.and_then(|stake_time| {
				current_perold.end.checked_sub(current_perold.start).and_then(|period_len| {
					balance_mul_div(stake_time.into(), amount, period_len.into())
				})
			})
			.and_then(|score| -> Option<(Balance, Balance)> {
				let new_score = pool_state.score.checked_add(score)?;
				let new_vote_amount = pool_state.total_amount.checked_add(amount)?;
				Some((new_score, new_vote_amount))
			})
			.ok_or(Error::<T>::Math)?;

		pool_state.score = calculate_result.0;
		pool_state.total_amount = calculate_result.1;

		AccountVoteAmount::<T>::try_mutate(who, pool_id, |mutatalbe_amount| -> DispatchResult {
			if let Some(old_amount) = mutatalbe_amount {
				*old_amount = old_amount.checked_add(amount).ok_or(Error::<T>::Math)?;
			} else {
				*mutatalbe_amount = Some(amount)
			}
			Ok(())
		})?;

		Self::deposit_event(Event::Vote {
			caller: who.clone(),
			period_id: current_period_id,
			pool_id,
			amount,
		});

		Ok(())
	}

	fn inner_cancel_vote(
		pool_state: &mut PoolState,
		who: &T::AccountId,
		amount: Balance,
		pool_id: T::PoolId,
		current_period_id: PeriodId,
		now: Timestamp,
	) -> DispatchResult {
		AccountVoteAmount::<T>::try_mutate(who, pool_id, |mutatalbe_amount| -> DispatchResult {
			if let Some(old_amount) = mutatalbe_amount {
				ensure!(*old_amount >= amount, Error::<T>::InsufficientAmount);

				*old_amount = old_amount.checked_sub(amount).ok_or(Error::<T>::Math)?;

				let pallet_account = T::PalletId::get().into_account_truncating();
				let vote_currency = Self::vote_currency().ok_or(Error::<T>::Uninitialized)?;
				T::MultiCurrency::transfer(vote_currency, &pallet_account, who, amount)?;

				let current_period =
					Self::vote_period(current_period_id).ok_or(Error::<T>::InvalidPeriodId)?;

				if now < current_period.start {
					pool_state.score =
						pool_state.score.checked_sub(amount).ok_or(Error::<T>::Math)?;
				} else if now <= current_period.end {
					pool_state.score = current_period
						.end
						.checked_sub(current_period.start)
						.and_then(|period_len| {
							balance_mul_div(
								(current_period.end - now).into(),
								amount,
								period_len.into(),
							)
						})
						.ok_or(Error::<T>::Math)?;
				}

				pool_state.total_amount =
					pool_state.total_amount.checked_sub(amount).ok_or(Error::<T>::Math)?;

				Self::deposit_event(Event::CancelVote {
					caller: who.clone(),
					period_id: current_period_id,
					pool_id,
					amount,
				});
				Ok(())
			} else {
				Err(Error::<T>::InsufficientAmount.into())
			}
		})?;

		Self::deposit_event(Event::Vote {
			caller: who.clone(),
			period_id: current_period_id,
			pool_id,
			amount,
		});

		Ok(())
	}

	fn inherit_expired_pool(
		pool_id: T::PoolId,
		pool_state: &mut PoolState,
		current_period_id: PeriodId,
	) -> DispatchResult {
		if current_period_id == 0 || pool_state.inherit {
			return Ok(())
		}

		let last_update_period_id = Self::pool_last_update_period(pool_id).unwrap_or_default();
		let last_pool_state = Self::global_pool_state(last_update_period_id, pool_id)
			.ok_or(Error::<T>::InvalidPeriodId)?;

		pool_state.inherit = true;
		pool_state.score = last_pool_state.total_amount;
		pool_state.total_amount = last_pool_state.total_amount;

		if !pool_state.reset_votable {
			pool_state.votable = last_pool_state.votable;
		}

		PoolLastUpdatePerold::<T>::mutate(pool_id, |last_update_period| {
			*last_update_period = Some(current_period_id)
		});

		Self::deposit_event(Event::InheritPool {
			pool_id,
			current_period_id,
			last_update_period_id,
			amount: pool_state.total_amount,
			votable: pool_state.votable,
		});
		Ok(())
	}
}
