#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as StablePallet;

use chain_primitives::{TokenSymbol::*, *};

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;

const UNIT: u128 = 1_000_000_000_000;

pub fn lookup_of_account<T: Config>(
	who: T::AccountId,
) -> <<T as frame_system::Config>::Lookup as StaticLookup>::Source {
	<T as frame_system::Config>::Lookup::unlookup(who)
}

benchmarks! {
	where_clause { where T::CurrencyId: From<CurrencyId> }

	create_base_pool{
		let admin_fee_receiver: T::AccountId = whitelisted_caller();

	}:_(RawOrigin::Root,
		[CurrencyId::Token(LOCAL1).into(), CurrencyId::Token(LOCAL2).into()].to_vec(),
		[1,1].to_vec(),
		10,
		10,
		10,
		admin_fee_receiver,
		[1,1].to_vec()
	)
}
