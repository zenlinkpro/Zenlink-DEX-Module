// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as ZenlinkPallet;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;

const UNIT: u128 = 1_000_000_000_000;

const ASSET_0: AssetId = AssetId {
	chain_id: 2001,
	asset_type: 2,
	asset_index: 770,
};

const ASSET_1: AssetId = AssetId {
	chain_id: 2001,
	asset_type: 2,
	asset_index: 516,
};

const ASSET_2: AssetId = AssetId {
	chain_id: 2001,
	asset_type: 2,
	asset_index: 518,
};

// alice_public_key
fn alice<T: Config>() -> T::AccountId {
	let mut public_key_data: [u8; 32] = [
		212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227,
		154, 86, 132, 231, 165, 109, 162, 125,
	];
	T::AccountId::decode(&mut &public_key_data[..]).unwrap_or_default()
}

// bob_public_key
fn bob<T: Config>() -> T::AccountId {
	let mut public_key_data:[u8;32] =[
		142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156,
		178, 38, 170, 71, 148, 242, 106, 72,
	];
	T::AccountId::decode(&mut &public_key_data[..]).unwrap_or_default()
}

fn limit_order_0<T: Config>() ->LimitOrder<T::BlockNumber, T::AccountId>{
	LimitOrder::<T::BlockNumber, T::AccountId>{
		maker: alice::<T>(),
		from_asset_id: ASSET_0,
		to_asset_id: ASSET_1,
		amount_in: 1 * UNIT,
		amount_out_min: 1 * UNIT,
		recipient: bob::<T>(),
		deadline: 1u128.saturated_into(),
		create_at: 0,
		signature: vec![
			176,  56,  37, 247, 208,  80, 247,  69,  88, 127, 246,
			142, 191, 212, 183,  67,  32,  61,  96,  26, 172, 225,
			21, 239, 185,  64, 227,   6,  25, 146, 164,  42, 108,
			94, 179,  11, 216, 197, 149,  21, 186,  24,  19,  19,
			235,  11, 149,  61, 168,  96,  45, 115,  14, 208, 223,
			151, 160,  36,   8,  18, 248, 133, 131, 135]
	}
}

pub fn lookup_of_account<T: Config>(
	who: T::AccountId,
) -> <<T as frame_system::Config>::Lookup as StaticLookup>::Source {
	<T as frame_system::Config>::Lookup::unlookup(who)
}

fn run_to_block<T: Config>(n: u32) {
	type System<T> = frame_system::Pallet<T>;

	while System::<T>::block_number() < n.saturated_into() {
		System::<T>::on_finalize(System::<T>::block_number());
		System::<T>::set_block_number(System::<T>::block_number() + 1u128.saturated_into());
		System::<T>::on_initialize(System::<T>::block_number());
	}
}

benchmarks! {

	set_fee_receiver{
		let caller: T::AccountId = whitelisted_caller();
	}:_(RawOrigin::Root, lookup_of_account::<T>(caller.clone()).into())

	set_fee_point{

	}:_(RawOrigin::Root, 5)

	create_pair {

	} : _(RawOrigin::Root, ASSET_0, ASSET_1)

	bootstrap_create {

	}: _(RawOrigin::Root, ASSET_0, ASSET_1, 1000, 1000, 1000_000_000, 1000_000_000, 100u128.saturated_into(), [ASSET_2].to_vec(),
		[(ASSET_0, 2000 * UNIT), (ASSET_1, 1000 * UNIT)].to_vec())

	bootstrap_contribute{
		let caller: T::AccountId = whitelisted_caller();

		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 1000 * UNIT));

		assert_ok!(ZenlinkPallet::<T>::bootstrap_create(
			(RawOrigin::Root).into(),
			ASSET_0,
			ASSET_1,
			1000,
			1000,
			1000_000_000,
			1000_000_000,
			100u128.saturated_into(),
			[ASSET_0].to_vec(),
			[(ASSET_0, 2 * UNIT), (ASSET_1, 1 * UNIT)].to_vec(),
		));

	}: _(RawOrigin::Signed(caller.clone()), ASSET_0, ASSET_1, UNIT, UNIT, 100u128.saturated_into())

	bootstrap_claim{
		let caller: T::AccountId = whitelisted_caller();

		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 2000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 2000 * UNIT));

		assert_ok!(ZenlinkPallet::<T>::bootstrap_create(
			(RawOrigin::Root).into(),
			ASSET_0,
			ASSET_1,
			1000,
			1000,
			10*UNIT,
			10*UNIT,
			99u128.saturated_into(),
			[ASSET_0].to_vec(),
			[(ASSET_0, 2 * UNIT), (ASSET_1, 1 * UNIT)].to_vec()
		));

		assert_ok!(ZenlinkPallet::<T>::bootstrap_charge_reward(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
			[(ASSET_0, 100 * UNIT)].to_vec()
		));

		assert_ok!(ZenlinkPallet::<T>::bootstrap_contribute(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
			10*UNIT,
			10*UNIT,
			99u128.saturated_into()
		));

		run_to_block::<T>(100);

		assert_ok!(ZenlinkPallet::<T>::bootstrap_end(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
		));

	}:_(RawOrigin::Signed(caller.clone()), lookup_of_account::<T>(caller.clone()), ASSET_0, ASSET_1, 120u128.saturated_into())

	bootstrap_end{
		let caller: T::AccountId = whitelisted_caller();

		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 1000 * UNIT));

		assert_ok!(ZenlinkPallet::<T>::bootstrap_create(
			(RawOrigin::Root).into(),
			ASSET_0,
			ASSET_1,
			1000,
			1000,
			10*UNIT,
			10*UNIT,
			99u128.saturated_into(),
			[ASSET_0].to_vec(),
			[(ASSET_0, 2 * UNIT)].to_vec()
		));

		assert_ok!(ZenlinkPallet::<T>::bootstrap_contribute(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
			10*UNIT,
			10*UNIT,
			99u128.saturated_into()
		));

		run_to_block::<T>(100);
	}:_(RawOrigin::Signed(caller.clone()), ASSET_0, ASSET_1)

	bootstrap_update{
		let caller: T::AccountId = whitelisted_caller();

		assert_ok!(ZenlinkPallet::<T>::bootstrap_create(
			(RawOrigin::Root).into(),
			ASSET_0,
			ASSET_1,
			1000,
			1000,
			10*UNIT,
			10*UNIT,
			99u128.saturated_into(),
			[ASSET_0, ASSET_1].to_vec(),
			[(ASSET_0, 2 * UNIT)].to_vec()
		));
	}:_(RawOrigin::Root, ASSET_0, ASSET_1, 1000, 1000, 1000_000_000, 1000_000_000, 100u128.saturated_into(),
		[ASSET_0, ASSET_1].to_vec(),
		[(ASSET_0, 2 * UNIT)].to_vec())

	bootstrap_refund{
		let caller: T::AccountId = whitelisted_caller();

		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 1000 * UNIT));

		assert_ok!(ZenlinkPallet::<T>::bootstrap_create(
			(RawOrigin::Root).into(),
			ASSET_0,
			ASSET_1,
			2*UNIT,
			2*UNIT,
			10*UNIT,
			10*UNIT,
			99u128.saturated_into(),
			[ASSET_0].to_vec(),
			[(ASSET_0, 2 * UNIT)].to_vec()
		));

		assert_ok!(ZenlinkPallet::<T>::bootstrap_contribute(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
			1*UNIT,
			1*UNIT,
			99u128.saturated_into()
		));
		run_to_block::<T>(100);
	}:_(RawOrigin::Signed(caller.clone()), ASSET_0, ASSET_1)

	add_liquidity{
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 1000 * UNIT));

		assert_ok!(ZenlinkPallet::<T>::create_pair((RawOrigin::Root).into(), ASSET_0, ASSET_1));

		assert_ok!(ZenlinkPallet::<T>::set_fee_receiver((RawOrigin::Root).into(), lookup_of_account::<T>(caller.clone()).into()));

	}:_(RawOrigin::Signed(caller.clone()), ASSET_0, ASSET_1, 10 * UNIT, 10* UNIT, 0,0, 100u32.saturated_into())

	remove_liquidity{
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 1000 * UNIT));

		assert_ok!(ZenlinkPallet::<T>::create_pair((RawOrigin::Root).into(), ASSET_0, ASSET_1));

		assert_ok!(ZenlinkPallet::<T>::set_fee_receiver((RawOrigin::Root).into(), lookup_of_account::<T>(caller.clone()).into()));

		assert_ok!(ZenlinkPallet::<T>::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
			10 * UNIT,
			10* UNIT,
			0,
			0,
			100u32.saturated_into()));

	}:_(RawOrigin::Signed(caller.clone()), ASSET_0, ASSET_1, 1 * UNIT, 0, 0, lookup_of_account::<T>(caller.clone()).into(), 100u32.saturated_into())

	swap_exact_assets_for_assets{
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_2, &caller, 1000 * UNIT));

		assert_ok!(ZenlinkPallet::<T>::create_pair((RawOrigin::Root).into(), ASSET_0, ASSET_1));
		assert_ok!(ZenlinkPallet::<T>::create_pair((RawOrigin::Root).into(), ASSET_1, ASSET_2));

		assert_ok!(ZenlinkPallet::<T>::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
			10 * UNIT,
			10* UNIT,
			0,
			0,
			100u32.saturated_into()));

		assert_ok!(ZenlinkPallet::<T>::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_1,
			ASSET_2,
			10 * UNIT,
			10* UNIT,
			0,
			0,
			100u32.saturated_into()));

		let path: Vec<AssetId> = vec![ASSET_0, ASSET_1, ASSET_2];

	}:_(RawOrigin::Signed(caller.clone()), 1* UNIT, 0,path, lookup_of_account::<T>(caller.clone()).into(), 100u32.saturated_into())

	swap_assets_for_exact_assets{
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_2, &caller, 1000 * UNIT));

		assert_ok!(ZenlinkPallet::<T>::create_pair((RawOrigin::Root).into(), ASSET_1, ASSET_2));
		assert_ok!(ZenlinkPallet::<T>::create_pair((RawOrigin::Root).into(), ASSET_0, ASSET_1));

		assert_ok!(ZenlinkPallet::<T>::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_1,
			ASSET_2,
			10 * UNIT,
			10* UNIT,
			0,
			0,
			100u32.saturated_into()));

		assert_ok!(ZenlinkPallet::<T>::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
			10 * UNIT,
			10* UNIT,
			0,
			0,
			100u32.saturated_into()));

		let path: Vec<AssetId> = vec![ASSET_0, ASSET_1, ASSET_2];
	}:_(RawOrigin::Signed(caller.clone()), 1* UNIT, 10*UNIT,path, lookup_of_account::<T>(caller.clone()).into(), 100u32.saturated_into())

	create_order{
		let caller: T::AccountId = whitelisted_caller();

		let order = limit_order_0::<T>();
	}:_(RawOrigin::Signed(caller.clone()), order)

	filled_order{
		let caller: T::AccountId = whitelisted_caller();

		let fill_order_args = FillOrderArgs::<T::BlockNumber, T::AccountId>{
			order: limit_order_0::<T>(),
			amount_to_fill_in: 1 * UNIT,
			path: vec![ASSET_0, ASSET_1]
		};

		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_0, &caller, 1000 * UNIT));
		assert_ok!(<T as Config>::MultiAssetsHandler::deposit(ASSET_1, &caller, 1000 * UNIT));
		assert_ok!(ZenlinkPallet::<T>::create_pair((RawOrigin::Root).into(), ASSET_0, ASSET_1));
		assert_ok!(ZenlinkPallet::<T>::add_liquidity(
			RawOrigin::Signed(caller.clone()).into(),
			ASSET_0,
			ASSET_1,
			100 * UNIT,
			200* UNIT,
			0,
			0,
			100u32.saturated_into())
		);

		ZenlinkPallet::<T>::create_order(RawOrigin::Signed(caller.clone()).into(), limit_order_0::<T>());

	}:_(RawOrigin::Signed(caller.clone()), fill_order_args)

	cancel_order{
		let caller: T::AccountId = whitelisted_caller();

		ZenlinkPallet::<T>::create_order(RawOrigin::Signed(caller.clone()).into(), limit_order_0::<T>());

		let order_hash = limit_order_0::<T>().hash().unwrap_or_default();

	}:_(RawOrigin::Signed(caller.clone()), order_hash)
}
