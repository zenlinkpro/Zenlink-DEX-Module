#![cfg_attr(not(feature = "std"), no_std)]

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

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}
