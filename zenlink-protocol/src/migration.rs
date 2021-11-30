use super::{Config, KLast, Weight};
use frame_support::traits::Get;
use sp_core::U256;

pub fn update_k_value_type_to_u256<T: Config>() -> Weight {
	KLast::<T>::translate_values::<u128, _>(|v| {
		if v == u128::MAX {
			Some(U256::zero())
		} else {
			Some(U256::from(v))
		}
	});

	T::DbWeight::get().reads(4) + T::DbWeight::get().writes(4)
}
