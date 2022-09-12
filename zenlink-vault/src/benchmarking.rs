#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as VaultPallet;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;

fn asset1<AssetId: TryFrom<u64> + Default>() -> AssetId {
	AssetId::try_from(513u64).unwrap_or_default()
}

pub fn lookup_of_account<T: Config>(
	who: T::AccountId,
) -> <<T as frame_system::Config>::Lookup as StaticLookup>::Source {
	<T as frame_system::Config>::Lookup::unlookup(who)
}

const UNIT: u128 = 1_000_000_000_000;

benchmarks! {
	where_clause { where T::AssetId: TryFrom<u64> + Default }

	create_vault_asset{}:_(RawOrigin::Root,asset1::<T::AssetId>(),12u8,12u8,500_000_000_000_000_000,0u128)

	deposit{
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(VaultPallet::<T>::create_vault_asset(
				RawOrigin::Root.into(),
				asset1::<T::AssetId>(),
				12u8,
				12u8,
				500_000_000_000_000_000,
				0u128)
		);

		assert_ok!(T::MultiAsset::deposit(asset1::<T::AssetId>(), &caller, UNIT * 1000));

	}:_(RawOrigin::Signed(caller.clone()), asset1::<T::AssetId>(), UNIT * 1,lookup_of_account::<T>(caller.clone()))

	mint{
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(VaultPallet::<T>::create_vault_asset(
			RawOrigin::Root.into(),
			asset1::<T::AssetId>(),
			12u8,
			12u8,
			500_000_000_000_000_000,
			0u128)
		);

		assert_ok!(T::MultiAsset::deposit(asset1::<T::AssetId>(), &caller, UNIT * 1000));

	}:_(RawOrigin::Signed(caller.clone()), asset1::<T::AssetId>(), UNIT * 1,lookup_of_account::<T>(caller.clone()))

	withdraw{
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(VaultPallet::<T>::create_vault_asset(
			RawOrigin::Root.into(),
			asset1::<T::AssetId>(),
			12u8,
			12u8,
			500_000_000_000_000_000,
			0u128)
		);

		assert_ok!(T::MultiAsset::deposit(asset1::<T::AssetId>(), &caller, UNIT * 1000));

		assert_ok!(VaultPallet::<T>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			asset1::<T::AssetId>(),
			UNIT * 1,
			lookup_of_account::<T>(caller.clone())
		));

	}:_(RawOrigin::Signed(caller.clone()), asset1::<T::AssetId>(), UNIT * 1,lookup_of_account::<T>(caller.clone()))

	redeem{
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(VaultPallet::<T>::create_vault_asset(
			RawOrigin::Root.into(),
			asset1::<T::AssetId>(),
			12u8,
			12u8,
			500_000_000_000_000_000,
			0u128)
		);

		assert_ok!(T::MultiAsset::deposit(asset1::<T::AssetId>(), &caller, UNIT * 1000));

		assert_ok!(VaultPallet::<T>::mint(
			RawOrigin::Signed(caller.clone()).into(),
			asset1::<T::AssetId>(),
			UNIT * 1,
			lookup_of_account::<T>(caller.clone())
		));

	}:_(RawOrigin::Signed(caller.clone()), asset1::<T::AssetId>(), UNIT * 1,lookup_of_account::<T>(caller.clone()))
}
