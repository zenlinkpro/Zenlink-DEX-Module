#![cfg(feature = "runtime-benchmarks")]

use super::{StableSwapMode::*, *};

use sp_std::vec;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;

use orml_traits::MultiCurrency;
use zenlink_protocol::{MultiAssetsHandler, Pallet as NormalAmmPallet};
use zenlink_stable_amm::Pallet as StableAmmPallet;

const UNIT: u128 = 1_000_000_000_000u128;

const INITIAL_A_VALUE: u128 = 50;
const SWAP_FEE: u128 = 10000000;
const ADMIN_FEE: u128 = 0;

const ASSET_0: AssetId = AssetId { chain_id: 2001, asset_type: 2, asset_index: 515 };
const ASSET_1: AssetId = AssetId { chain_id: 2001, asset_type: 2, asset_index: 514 };

fn token1<CurrencyId: TryFrom<u64> + Default>() -> CurrencyId {
	CurrencyId::try_from(513u64).unwrap_or_default()
}

fn token2<CurrencyId: TryFrom<u64> + Default>() -> CurrencyId {
	CurrencyId::try_from(514u64).unwrap_or_default()
}

benchmarks! {
	where_clause { where T: Config + zenlink_protocol::Config + zenlink_stable_amm::Config,
						<T as zenlink_stable_amm::Config>::CurrencyId: TryFrom<u64> + Default,
						<T as Config>::CurrencyId: TryFrom<u64> + Default
	}

	swap_exact_token_for_tokens_through_stable_pool{
		let caller: T::AccountId = whitelisted_caller();

		assert_ok!(<T as zenlink_protocol::Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 1000 * UNIT));
		assert_ok!(<T as zenlink_protocol::Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 1000 * UNIT));

		let stable_token1 = token1::<<T as zenlink_stable_amm::Config>::CurrencyId>();
		let stable_token2 = token2::<<T as zenlink_stable_amm::Config>::CurrencyId>();

		assert_ok!(<T as zenlink_stable_amm::Config>::MultiCurrency::deposit(stable_token1, &caller, 1000 * UNIT));
		assert_ok!(<T as zenlink_stable_amm::Config>::MultiCurrency::deposit(stable_token2, &caller, 1000 * UNIT));

		assert_ok!(StableAmmPallet::<T>::create_base_pool(
			(RawOrigin::Root).into(),
			[stable_token1, stable_token2].to_vec(),
			[12,12].to_vec(),
			INITIAL_A_VALUE,
			SWAP_FEE,
			ADMIN_FEE,
			caller.clone(),
			Vec::from("stable_pool_lp_0")
		));

		assert_ok!(NormalAmmPallet::<T>::create_pair(
			(RawOrigin::Root).into(),
			ASSET_0,
			ASSET_1,
		));


		assert_ok!(NormalAmmPallet::<T>::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
			100 * UNIT,
			100 * UNIT,
			0,
			0,
			100u32.into()
		));

		assert_ok!(StableAmmPallet::<T>::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			0u32.into(),
			[10*UNIT, 10*UNIT].to_vec(),
			0,
			caller.clone(),
			1000u32.into()
		));

		let router_stable_token1 = token1::<<T as Config>::CurrencyId>();
		let router_stable_token2 = token2::<<T as Config>::CurrencyId>();

	 }:_(
		RawOrigin::Signed(caller.clone()),
		(100u32).into(),
		0u32.into(),
		vec![
			Route::Normal([ASSET_1, ASSET_0].to_vec()),
			Route::Stable(StablePath::<T::StablePoolId, <T as Config>::CurrencyId> {
				pool_id: 0u32.into(),
				base_pool_id: 0u32.into(),
				mode: Single,
				from_currency: router_stable_token2,
				to_currency: router_stable_token1,
			}),
		],
		caller.clone(),
		1000u32.into()
	)
}
