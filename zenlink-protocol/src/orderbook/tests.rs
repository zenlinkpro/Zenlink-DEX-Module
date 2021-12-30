// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! Test utilities

use super::*;
use crate::orderbook::LimitOrder;
use crate::DispatchError::Other;
use frame_support::{assert_noop, assert_ok};
use sp_core::crypto::AccountId32;

use super::{mock::*, AssetId, Error};

const TEST_ASSET_0: AssetId = AssetId {
	chain_id: 200,
	asset_type: LOCAL,
	asset_index: 2,
};

const TEST_ASSET_1: AssetId = AssetId {
	chain_id: 200,
	asset_type: LOCAL,
	asset_index: 3,
};

const UNIT: u128 = 1_000_000_000_000;

fn limit_order_0() -> LimitOrder<u64, AccountId32> {
	LimitOrder::<u64, AccountId32> {
		maker: alice(),
		from_asset_id: TEST_ASSET_0,
		to_asset_id: TEST_ASSET_1,
		amount_in: 1 * UNIT,
		amount_out_min: 1 * UNIT,
		recipient: bob(),
		deadline: 10,
		create_at: 0,
		amount_filled_out: 0,
		signature: vec![
			10, 84, 151, 19, 56, 162, 25, 244, 152, 246, 6, 114, 171, 14, 158, 106, 136, 196, 88, 245, 246, 245, 171,
			204, 146, 172, 90, 115, 114, 157, 203, 62, 8, 139, 13, 20, 219, 229, 25, 6, 176, 150, 57, 170, 20, 38, 242,
			58, 103, 147, 97, 134, 223, 180, 121, 92, 15, 201, 10, 109, 190, 29, 11, 131,
		],
	}
}

#[test]
fn create_order_should_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order = limit_order_0();
		assert_ok!(DexPallet::inner_create_order(&mut limit_order));

		let order_hash = limit_order.hash().unwrap_or_default();
		assert_eq!(DexPallet::get_all_order_hash(), vec![order_hash]);
		assert_eq!(DexPallet::get_order_hash_of_maker(alice()), vec![order_hash]);
		assert_eq!(
			DexPallet::get_order_hash_of_from_asset(limit_order.from_asset_id),
			vec![order_hash]
		);
		assert_eq!(
			DexPallet::get_order_hash_of_to_asset(limit_order.to_asset_id),
			vec![order_hash]
		);
		assert_eq!(DexPallet::get_order_of_hash(order_hash), limit_order);
		assert_eq!(DexPallet::get_canceled_of_hash(alice()).get(&order_hash), None);
		assert_eq!(DexPallet::get_amount_fill_in_of_hash(&order_hash), 0u128);
	})
}

#[test]
fn create_same_order_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order = limit_order_0();

		assert_ok!(DexPallet::inner_create_order(&mut limit_order));
		assert_noop!(
			DexPallet::inner_create_order(&mut limit_order),
			Error::<Test>::LimitOrderAlreadyExist
		);

		let order_hash = limit_order.hash().unwrap_or_default();
		assert_eq!(DexPallet::get_all_order_hash(), vec![order_hash]);
		assert_eq!(DexPallet::get_order_hash_of_maker(alice()), vec![order_hash]);
		assert_eq!(
			DexPallet::get_order_hash_of_from_asset(limit_order.from_asset_id),
			vec![order_hash]
		);
		assert_eq!(
			DexPallet::get_order_hash_of_to_asset(limit_order.to_asset_id),
			vec![order_hash]
		);
		assert_eq!(DexPallet::get_order_of_hash(order_hash), limit_order);
		assert_eq!(DexPallet::get_canceled_of_hash(alice()).get(&order_hash), None);
		assert_eq!(DexPallet::get_amount_fill_in_of_hash(&order_hash), 0u128);
	})
}

#[test]
fn create_order_not_sign_by_maker_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order_with_mismatch_maker = limit_order_0();
		limit_order_with_mismatch_maker.signature = vec![
			108, 55, 107, 34, 10, 181, 247, 106, 148, 139, 190, 202, 94, 182, 40, 174, 127, 169, 107, 45, 202, 223,
			195, 213, 102, 223, 250, 211, 70, 104, 45, 19, 28, 89, 96, 240, 70, 183, 63, 28, 193, 104, 151, 22, 125,
			67, 109, 14, 52, 239, 185, 95, 8, 31, 244, 218, 42, 178, 237, 106, 201, 133, 17, 139,
		];
		// the signature is sign by bob.
		assert_noop!(
			DexPallet::inner_create_order(&mut limit_order_with_mismatch_maker),
			Error::<Test>::InvalidSignature
		);

		let order_hash = limit_order_with_mismatch_maker.hash().unwrap_or_default();
		assert_eq!(DexPallet::get_all_order_hash(), vec![]);
		assert_eq!(DexPallet::get_order_hash_of_maker(bob()), vec![]);
		assert_eq!(
			DexPallet::get_order_hash_of_from_asset(limit_order_with_mismatch_maker.from_asset_id),
			vec![]
		);
		assert_eq!(
			DexPallet::get_order_hash_of_to_asset(limit_order_with_mismatch_maker.to_asset_id),
			vec![]
		);
		assert_eq!(DexPallet::get_order_of_hash(order_hash), LimitOrder::default());
	})
}

#[test]
fn create_order_with_invalid_maker_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order_with_mismatch_maker = limit_order_0();
		limit_order_with_mismatch_maker.maker = AccountId32::default();
		assert_eq!(
			DexPallet::inner_create_order(&mut limit_order_with_mismatch_maker),
			Err(Other("invalid maker"))
		);

		let order_hash = limit_order_with_mismatch_maker.hash().unwrap_or_default();
		assert_eq!(DexPallet::get_all_order_hash(), vec![]);
		assert_eq!(DexPallet::get_order_hash_of_maker(bob()), vec![]);
		assert_eq!(
			DexPallet::get_order_hash_of_from_asset(limit_order_with_mismatch_maker.from_asset_id),
			vec![]
		);
		assert_eq!(
			DexPallet::get_order_hash_of_to_asset(limit_order_with_mismatch_maker.to_asset_id),
			vec![]
		);
		assert_eq!(DexPallet::get_order_of_hash(order_hash), LimitOrder::default());
	})
}

#[test]
fn create_order_with_invalid_from_asset_id_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order_with_mismatch_from_asset_id = limit_order_0();
		limit_order_with_mismatch_from_asset_id.from_asset_id = AssetId::default();
		assert_eq!(
			DexPallet::inner_create_order(&mut limit_order_with_mismatch_from_asset_id),
			Err(Other("invalid from_asset_id"))
		);
	})
}

#[test]
fn create_order_with_invalid_to_asset_id_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order_with_mismatch_to_asset_id = limit_order_0();
		limit_order_with_mismatch_to_asset_id.to_asset_id = AssetId::default();
		assert_eq!(
			DexPallet::inner_create_order(&mut limit_order_with_mismatch_to_asset_id),
			Err(Other("invalid to_asset_id"))
		);
	})
}

#[test]
fn create_order_with_duplicate_asset_id_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order_with_mismatch_to_asset_id = limit_order_0();
		limit_order_with_mismatch_to_asset_id.to_asset_id = limit_order_with_mismatch_to_asset_id.from_asset_id;
		assert_eq!(
			DexPallet::inner_create_order(&mut limit_order_with_mismatch_to_asset_id),
			Err(Other("duplicate-tokens"))
		);
	})
}

#[test]
fn create_order_with_invalid_amount_in_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order_with_mismatch_amount_in = limit_order_0();
		limit_order_with_mismatch_amount_in.amount_in = 0u128;
		assert_eq!(
			DexPallet::inner_create_order(&mut limit_order_with_mismatch_amount_in),
			Err(Other("invalid amount_in"))
		);
	})
}

#[test]
fn create_order_with_invalid_recipient_min_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order_with_mismatch_amount_out_min = limit_order_0();
		limit_order_with_mismatch_amount_out_min.recipient = AccountId32::default();
		assert_eq!(
			DexPallet::inner_create_order(&mut limit_order_with_mismatch_amount_out_min),
			Err(Other("invalid recipient"))
		);
	})
}

#[test]
fn create_order_with_invalid_amount_out_min_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order_with_mismatch_amount_out_min = limit_order_0();
		limit_order_with_mismatch_amount_out_min.amount_out_min = 0;
		assert_eq!(
			DexPallet::inner_create_order(&mut limit_order_with_mismatch_amount_out_min),
			Err(Other("invalid amount_out_min"))
		);
	})
}

#[test]
fn create_order_with_invalid_deadline_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut limit_order_with_mismatch_deadline = limit_order_0();
		limit_order_with_mismatch_deadline.deadline = 0;
		assert_eq!(
			DexPallet::inner_create_order(&mut limit_order_with_mismatch_deadline),
			Err(Other("invalid deadline"))
		);
	})
}

fn create_pair(asset_0: AssetId, asset_1: AssetId, amount_0: u128, amount_1: u128) -> DispatchResult {
	DexPallet::foreign_mint(asset_0, &alice(), u128::MAX)?;
	DexPallet::foreign_mint(asset_1, &alice(), u128::MAX)?;

	DexPallet::create_pair(Origin::root(), asset_0, asset_1)?;

	DexPallet::add_liquidity(Origin::signed(alice()), asset_0, asset_1, amount_0, amount_1, 0, 0, 100)
}

fn calculate_swap_amount_out(
	input_amount: AssetBalance,
	input_reserve: AssetBalance,
	output_reserve: AssetBalance,
) -> AssetBalance {
	if input_reserve.is_zero() || output_reserve.is_zero() || input_amount.is_zero() {
		return Zero::zero();
	}

	let input_amount_with_fee = U256::from(input_amount).saturating_mul(U256::from(997));

	let numerator = input_amount_with_fee.saturating_mul(U256::from(output_reserve));

	let denominator = U256::from(input_reserve)
		.saturating_mul(U256::from(1000))
		.saturating_add(input_amount_with_fee);

	numerator
		.checked_div(denominator)
		.and_then(|n| TryInto::<AssetBalance>::try_into(n).ok())
		.unwrap_or_else(Zero::zero)
}

#[test]
fn fill_order_should_work() {
	new_test_ext().execute_with(|| {
		let total_supply_0 = 1000 * UNIT;
		let total_supply_1 = 1100 * UNIT;
		assert_ok!(create_pair(TEST_ASSET_0, TEST_ASSET_1, total_supply_0, total_supply_1));

		let mut order = limit_order_0();
		assert_ok!(DexPallet::inner_create_order(&mut order));

		let fill_in_amount = 1 * UNIT;
		let mut fill_order_args = FillOrderArgs::<u64, AccountId32>::default();
		fill_order_args.order = order;
		fill_order_args.amount_to_fill_in = fill_in_amount;
		fill_order_args.path = vec![fill_order_args.order.from_asset_id, fill_order_args.order.to_asset_id];

		let order_hash = fill_order_args.order.hash().unwrap_or_default();

		assert_ok!(DexPallet::filled_order(Origin::signed(alice()), fill_order_args));

		let bob_test_asset_0_balance = <Test as Config>::MultiAssetsHandler::balance_of(TEST_ASSET_1, &bob());
		assert_eq!(
			bob_test_asset_0_balance,
			calculate_swap_amount_out(fill_in_amount, total_supply_0, total_supply_1)
		);

		assert_eq!(DexPallet::get_canceled_of_hash(alice()).get(&order_hash), None);
		assert_eq!(DexPallet::get_amount_fill_in_of_hash(&order_hash), fill_in_amount);
		let order_after_filled = DexPallet::get_order_of_hash(order_hash);
		assert_eq!(order_after_filled.amount_filled_out, bob_test_asset_0_balance)
	})
}

#[test]
fn fill_already_filed_order_should_not_work() {
	new_test_ext().execute_with(|| {
		let total_supply_0 = 1000 * UNIT;
		let total_supply_1 = 1100 * UNIT;
		assert_ok!(create_pair(TEST_ASSET_0, TEST_ASSET_1, total_supply_0, total_supply_1));

		let mut order = limit_order_0();
		assert_ok!(DexPallet::inner_create_order(&mut order));

		let fill_in_amount = 1 * UNIT;
		let mut fill_order_args = FillOrderArgs::<u64, AccountId32>::default();
		fill_order_args.order = order;
		fill_order_args.amount_to_fill_in = fill_in_amount;
		fill_order_args.path = vec![fill_order_args.order.from_asset_id, fill_order_args.order.to_asset_id];

		let order_hash = fill_order_args.order.hash().unwrap_or_default();

		assert_ok!(DexPallet::filled_order(
			Origin::signed(alice()),
			fill_order_args.clone()
		));
		assert_eq!(
			DexPallet::filled_order(Origin::signed(alice()), fill_order_args),
			Err(Other("order already filled"))
		);

		let bob_test_asset_0_balance = <Test as Config>::MultiAssetsHandler::balance_of(TEST_ASSET_1, &bob());
		assert_eq!(
			bob_test_asset_0_balance,
			calculate_swap_amount_out(fill_in_amount, total_supply_0, total_supply_1)
		);

		assert_eq!(DexPallet::get_canceled_of_hash(alice()).get(&order_hash), None);
		assert_eq!(DexPallet::get_amount_fill_in_of_hash(&order_hash), fill_in_amount);
		let order_after_filled = DexPallet::get_order_of_hash(order_hash);
		assert_eq!(order_after_filled.amount_filled_out, bob_test_asset_0_balance)
	})
}

#[test]
fn fill_order_with_invalid_signature_should_not_work() {
	new_test_ext().execute_with(|| {
		let total_supply_0 = 1000 * UNIT;
		let total_supply_1 = 1100 * UNIT;
		assert_ok!(create_pair(TEST_ASSET_0, TEST_ASSET_1, total_supply_0, total_supply_1));

		let mut order = limit_order_0();
		assert_ok!(DexPallet::inner_create_order(&mut order));

		let fill_in_amount = 1 * UNIT;
		let mut fill_order_args = FillOrderArgs::<u64, AccountId32>::default();
		fill_order_args.order = order;
		fill_order_args.order.maker = bob();
		fill_order_args.amount_to_fill_in = fill_in_amount;
		fill_order_args.path = vec![fill_order_args.order.from_asset_id, fill_order_args.order.to_asset_id];

		let order_hash = fill_order_args.order.hash().unwrap_or_default();

		assert_eq!(
			DexPallet::filled_order(Origin::signed(alice()), fill_order_args),
			Err(Other("order not exist"))
		);

		let bob_test_asset_0_balance = <Test as Config>::MultiAssetsHandler::balance_of(TEST_ASSET_1, &bob());
		assert_eq!(bob_test_asset_0_balance, 0u128);

		assert_eq!(DexPallet::get_amount_fill_in_of_hash(&order_hash), 0u128);
	})
}

#[test]
fn cancel_order_should_work() {
	new_test_ext().execute_with(|| {
		let mut order = limit_order_0();
		assert_ok!(DexPallet::inner_create_order(&mut order));

		let order_hash = order.hash().unwrap_or_default();

		assert_ok!(DexPallet::cancel_order(Origin::signed(alice()), order_hash,));
	})
}

#[test]
fn cancel_not_exist_order_should_not_work() {
	new_test_ext().execute_with(|| {
		let mut order = limit_order_0();
		let order_hash = order.hash().unwrap_or_default();

		assert_eq!(
			DexPallet::cancel_order(Origin::signed(alice()), order_hash,),
			Err(Other("order not exist"))
		);
	})
}
