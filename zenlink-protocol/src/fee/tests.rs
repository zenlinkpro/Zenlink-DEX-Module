// Copyright 2021-2022 Zenlink.
// Licensed under Apache 2.0.

use super::mock::*;
use crate::{AssetId, MultiAssetsHandler};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use frame_system::RawOrigin;
use sp_core::U256;

const DOT_ASSET_ID: AssetId = AssetId { chain_id: 200, asset_type: LOCAL, asset_index: 2 };

const BTC_ASSET_ID: AssetId = AssetId { chain_id: 300, asset_type: RESERVED, asset_index: 3 };

const PAIR_DOT_BTC: u128 = 111825939709248857954450132390071529325;

const ALICE: u128 = 1;
const BOB: u128 = 2;
const CHARLIE: u128 = 3;
const DOT_UNIT: u128 = 1000_000_000_000_000;
const BTC_UNIT: u128 = 1000_000_00;
const LP_DOT_BTC: AssetId = AssetId { chain_id: 0, asset_type: 2, asset_index: 12885034496 };

#[test]
fn fee_meta_getter_should_work() {
	new_test_ext().execute_with(|| {
		let (fee_receiver, fee_point) = DexPallet::fee_meta();

		assert_eq!(fee_receiver, None);
		assert_eq!(fee_point, 5);
	})
}

#[test]
fn fee_meta_setter_should_not_work() {
	new_test_ext().execute_with(|| {
		let (fee_receiver, fee_point) = DexPallet::fee_meta();

		assert_eq!(fee_receiver, None);
		assert_eq!(fee_point, 5);

		assert_noop!(
			DexPallet::set_fee_receiver(RawOrigin::Signed(BOB).into(), Some(BOB)),
			BadOrigin,
		);

		assert_noop!(DexPallet::set_fee_point(RawOrigin::Signed(BOB).into(), 0), BadOrigin);
	})
}

#[test]
fn turn_on_protocol_fee_only_add_liquidity_no_fee_should_work() {
	new_test_ext().execute_with(|| {
		// 1. turn on the protocol fee
		// use default rate: 1/6

		let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);

		assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));
		assert_eq!(DexPallet::k_last(sorted_pair), U256::zero());

		// 2. first add_liquidity

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

		let total_supply_dot: u128 = 1 * DOT_UNIT;
		let total_supply_btc: u128 = 1 * BTC_UNIT;

		assert_ok!(DexPallet::create_pair(RawOrigin::Root.into(), DOT_ASSET_ID, BTC_ASSET_ID,));

		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0,
			100
		));

		let lp_of_alice_0 = 316227766016;
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_0
		);
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), 0);
		assert_eq!(DexPallet::k_last(sorted_pair), U256::from(DOT_UNIT) * U256::from(BTC_UNIT));

		// 3. second add_liquidity

		let total_supply_dot = 50 * DOT_UNIT;
		let total_supply_btc = 50 * BTC_UNIT;

		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			BTC_ASSET_ID,
			DOT_ASSET_ID,
			total_supply_btc,
			total_supply_dot,
			0,
			0,
			100
		));

		let lp_of_alice_1 = 16127616066816u128;
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_1
		);
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), 0);
		assert_eq!(
			DexPallet::k_last(sorted_pair),
			U256::from(51 * DOT_UNIT) * U256::from(51 * BTC_UNIT)
		);

		let balance_dot =
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &PAIR_DOT_BTC);
		let balance_btc =
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &PAIR_DOT_BTC);

		//println!("balance_DOT {}, balance_BTC {}", balance_dot, balance_btc);
		assert_eq!(balance_dot, 51000000000000000);
		assert_eq!(balance_btc, 5100000000);
		assert_eq!((balance_dot / DOT_UNIT), (balance_btc / BTC_UNIT));

		// 4. third add_liquidity
		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			BTC_ASSET_ID,
			DOT_ASSET_ID,
			total_supply_btc,
			total_supply_dot,
			0,
			0,
			100
		));

		let lp_total = <Test as Config>::MultiAssetsHandler::total_supply(LP_DOT_BTC);
		let lp_of_alice_2 = 31939004367616u128;
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_2
		);
		let lp_of_bob = 0u128;
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), lp_of_bob);
		assert_eq!(lp_total, lp_of_alice_2 + lp_of_bob);

		assert_eq!(
			DexPallet::k_last(sorted_pair),
			U256::from(101 * DOT_UNIT) * U256::from(101 * BTC_UNIT)
		);
	});
}

#[test]
fn turn_on_protocol_fee_remove_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		// 1. turn on the protocol fee
		// use default rate: 1/6

		let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);

		assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));
		assert_eq!(DexPallet::k_last(sorted_pair), U256::zero());

		// 2. first add_liquidity

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, 10000 * DOT_UNIT));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, 10000 * BTC_UNIT));

		let total_supply_dot: u128 = 1 * DOT_UNIT;
		let total_supply_btc: u128 = 1 * BTC_UNIT;

		assert_ok!(DexPallet::create_pair(RawOrigin::Root.into(), DOT_ASSET_ID, BTC_ASSET_ID,));

		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0,
			100
		));

		let lp_of_alice_0 = 316227766016;
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			316227766016
		);
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), 0);
		assert_eq!(DexPallet::k_last(sorted_pair), U256::from(DOT_UNIT) * U256::from(BTC_UNIT));

		// 3. second add_liquidity

		let total_supply_dot = 50 * DOT_UNIT;
		let total_supply_btc = 50 * BTC_UNIT;

		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			BTC_ASSET_ID,
			DOT_ASSET_ID,
			total_supply_btc,
			total_supply_dot,
			0,
			0,
			100
		));

		let lp_of_alice_1 = 16127616066816u128;
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_1
		);
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), 0);
		assert_eq!(
			DexPallet::k_last(sorted_pair),
			U256::from(51 * DOT_UNIT) * U256::from(51 * BTC_UNIT)
		);

		let balance_dot =
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &PAIR_DOT_BTC);
		let balance_btc =
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &PAIR_DOT_BTC);

		//println!("balance_DOT {}, balance_BTC {}", balance_dot, balance_btc);
		assert_eq!(balance_dot, 51000000000000000);
		assert_eq!(balance_btc, 5100000000);
		assert_eq!((balance_dot / DOT_UNIT), (balance_btc / BTC_UNIT));

		// 4. remove_liquidity
		assert_ok!(DexPallet::remove_liquidity(
			RawOrigin::Signed(ALICE).into(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			lp_of_alice_0,
			0u128,
			0u128,
			ALICE,
			100
		));

		let lp_total = <Test as Config>::MultiAssetsHandler::total_supply(LP_DOT_BTC);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_1 - lp_of_alice_0
		);
		let lp_of_bob = 0u128;
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), lp_of_bob);
		assert_eq!(lp_total, lp_of_alice_1 - lp_of_alice_0 + lp_of_bob);
		assert_eq!(
			DexPallet::k_last(sorted_pair),
			U256::from(50 * DOT_UNIT) * U256::from(50 * BTC_UNIT)
		);
	});
}

#[test]
fn turn_on_protocol_fee_swap_have_fee_should_work() {
	new_test_ext().execute_with(|| {
		// 1. turn on the protocol fee

		let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);

		assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));
		// use default rate: 0.3% * 1 / 6 = 0.0005
		assert_ok!(DexPallet::set_fee_point(RawOrigin::Root.into(), 5u8));
		assert_eq!(DexPallet::k_last(sorted_pair), U256::zero());

		// 2. first add_liquidity

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, DOT_UNIT * 1000));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, BTC_UNIT * 1000));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &CHARLIE, DOT_UNIT * 1000));

		assert_ok!(DexPallet::create_pair(RawOrigin::Root.into(), DOT_ASSET_ID, BTC_ASSET_ID,));

		let total_supply_dot: u128 = 1 * DOT_UNIT;
		let total_supply_btc: u128 = 1 * BTC_UNIT;

		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0,
			100
		));

		let lp_of_alice_0 = 316227766016;
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_0
		);
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), 0);
		assert_eq!(DexPallet::k_last(sorted_pair), U256::from(DOT_UNIT) * U256::from(BTC_UNIT));

		// 3. swap

		assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
			&CHARLIE,
			DOT_UNIT,
			1,
			&vec![DOT_ASSET_ID, BTC_ASSET_ID],
			&CHARLIE,
		));

		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_0
		);
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), 0);
		assert_eq!(DexPallet::k_last(sorted_pair), U256::from(DOT_UNIT) * U256::from(BTC_UNIT));

		let balance_dot =
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &PAIR_DOT_BTC);
		let balance_btc =
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &PAIR_DOT_BTC);

		//println!("balance_DOT {}, balance_BTC {}", balance_dot, balance_btc);
		assert_eq!(balance_dot, 2000000000000000);
		assert_eq!(balance_btc, 50075113);

		let k_last = DexPallet::k_last(sorted_pair);
		let reserve_0 =
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &PAIR_DOT_BTC);
		let reserve_1 =
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &PAIR_DOT_BTC);
		let root_k = U256::from(reserve_0).saturating_mul(U256::from(reserve_1)).integer_sqrt();
		let root_k_last = k_last.integer_sqrt();

		assert!(root_k > root_k_last);

		let lp_total = <Test as Config>::MultiAssetsHandler::total_supply(LP_DOT_BTC);
		let numerator = U256::from(lp_total).saturating_mul(root_k.saturating_sub(root_k_last));
		let denominator = root_k.saturating_mul(U256::from(5u128)).saturating_add(root_k_last);
		let expect_fee = numerator.checked_div(denominator).unwrap_or_default();

		// 4. second add_liquidity
		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			BTC_ASSET_ID,
			DOT_ASSET_ID,
			total_supply_btc,
			total_supply_dot,
			0,
			0,
			100
		));

		let lp_total = <Test as Config>::MultiAssetsHandler::total_supply(LP_DOT_BTC);
		let lp_of_alice_2 = 474361420078u128;
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_2
		);

		let lp_of_bob = 39548424u128;
		assert_eq!(expect_fee, U256::from(lp_of_bob));
		assert_eq!(
			U256::from(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB)),
			expect_fee
		);
		assert_eq!(lp_total, lp_of_alice_2 + lp_of_bob);

		assert_eq!(DexPallet::k_last(sorted_pair), U256::from(225338007000000000000000u128));
	});
}

#[test]
fn turn_on_protocol_fee_swap_have_fee_at_should_work() {
	new_test_ext().execute_with(|| {
		// 1. turn on the protocol fee

		let sorted_pair = DexPallet::sort_asset_id(DOT_ASSET_ID, BTC_ASSET_ID);

		assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));
		// use default rate: 0.3% * 1 / 6 = 0.0005
		assert_ok!(DexPallet::set_fee_point(RawOrigin::Root.into(), 5u8));
		assert_eq!(DexPallet::k_last(sorted_pair), U256::zero());

		// 2. first add_liquidity

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, 100_000_000 * DOT_UNIT));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, 100_000_000 * BTC_UNIT));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &CHARLIE, 100_000_000 * DOT_UNIT));

		assert_ok!(DexPallet::create_pair(RawOrigin::Root.into(), DOT_ASSET_ID, BTC_ASSET_ID,));

		let total_supply_dot: u128 = 1_000_000 * DOT_UNIT;
		let total_supply_btc: u128 = 1_000_000 * BTC_UNIT;

		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0,
			100
		));

		let lp_of_alice_0 = U256::from(total_supply_btc)
			.saturating_mul(U256::from(total_supply_dot))
			.integer_sqrt()
			.as_u128();
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_0
		);
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), 0);
		assert_eq!(
			DexPallet::k_last(sorted_pair),
			U256::from(total_supply_btc) * U256::from(total_supply_dot)
		);

		// 3. swap

		assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
			&CHARLIE,
			DOT_UNIT,
			1,
			&vec![DOT_ASSET_ID, BTC_ASSET_ID],
			&CHARLIE,
		));

		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_0
		);
		//println!("{:#?}", lp_of_alice_0);
		assert_eq!(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB), 0);
		assert_eq!(
			DexPallet::k_last(sorted_pair),
			U256::from(total_supply_btc) * U256::from(total_supply_dot)
		);

		let k_last = DexPallet::k_last(sorted_pair);
		let reserve_0 =
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &PAIR_DOT_BTC);
		let reserve_1 =
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &PAIR_DOT_BTC);

		assert_eq!(reserve_0, total_supply_dot + 1 * DOT_UNIT);
		assert_eq!(reserve_1, total_supply_btc - 99699900);

		let root_k = U256::from(reserve_0).saturating_mul(U256::from(reserve_1)).integer_sqrt();
		let root_k_last = k_last.integer_sqrt();

		assert!(root_k > root_k_last);

		let lp_total = <Test as Config>::MultiAssetsHandler::total_supply(LP_DOT_BTC);
		let numerator = U256::from(lp_total).saturating_mul(root_k.saturating_sub(root_k_last));
		let denominator = root_k.saturating_mul(U256::from(5u128)).saturating_add(root_k_last);
		let expect_fee = numerator.checked_div(denominator).unwrap_or_default();

		let (added_btc, _) = DexPallet::calculate_added_amount(
			1 * BTC_UNIT,
			1 * DOT_UNIT,
			0,
			0,
			reserve_1,
			reserve_0,
		)
		.unwrap();

		// // 4. second add_liquidity
		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			BTC_ASSET_ID,
			DOT_ASSET_ID,
			1 * BTC_UNIT,
			1 * DOT_UNIT,
			0,
			0,
			100
		));

		let lp_fee = <Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB);

		let alice_lp_add = (U256::from(lp_of_alice_0 + lp_fee) * U256::from(added_btc) /
			U256::from(reserve_1))
		.as_u128();

		let lp_total = <Test as Config>::MultiAssetsHandler::total_supply(LP_DOT_BTC);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &ALICE),
			lp_of_alice_0 + alice_lp_add
		);

		let lp_of_bob = <Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB);
		assert_eq!(expect_fee, U256::from(lp_of_bob));
		assert_eq!(
			U256::from(<Test as Config>::MultiAssetsHandler::balance_of(LP_DOT_BTC, &BOB)),
			expect_fee
		);
		assert_eq!(lp_total, lp_of_alice_0 + alice_lp_add + lp_of_bob);
	});
}

#[test]
fn assert_higher_fee_point_decreases_protocol_fee() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexPallet::set_fee_receiver(RawOrigin::Root.into(), Some(BOB)));

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, 100_000_000 * DOT_UNIT));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, 100_000_000 * BTC_UNIT));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &CHARLIE, 100_000_000 * DOT_UNIT));

		assert_ok!(DexPallet::create_pair(RawOrigin::Root.into(), DOT_ASSET_ID, BTC_ASSET_ID,));

		let total_supply_dot: u128 = 1_000_000 * DOT_UNIT;
		let total_supply_btc: u128 = 1_000_000 * BTC_UNIT;

		assert_ok!(DexPallet::add_liquidity(
			RawOrigin::Signed(ALICE).into(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0,
			100
		));

		assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
			&CHARLIE,
			DOT_UNIT,
			1,
			&vec![DOT_ASSET_ID, BTC_ASSET_ID],
			&CHARLIE,
		));

		fn mint_fee_for_point(fee_point: u8) -> u128 {
			let reserve_0 =
				<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &PAIR_DOT_BTC);
			let reserve_1 =
				<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &PAIR_DOT_BTC);
			let total_supply = <Test as Config>::MultiAssetsHandler::total_supply(LP_DOT_BTC);
			assert_ok!(DexPallet::set_fee_point(RawOrigin::Root.into(), fee_point));
			DexPallet::mint_protocol_fee(
				reserve_0,
				reserve_1,
				DOT_ASSET_ID,
				BTC_ASSET_ID,
				total_supply,
			)
			.unwrap()
		}

		// 1/(1/1)-1=0
		let total_fee = mint_fee_for_point(0);
		// 1/(1/2)-1=1
		assert_eq!(mint_fee_for_point(1), total_fee / 2);
		// 1/(1/4)-1=3
		assert_eq!(mint_fee_for_point(3), total_fee / 4);
		// 1/(1/6)-1=5
		assert_eq!(mint_fee_for_point(5), total_fee / 6);
		// 1/(1/10)-1=9
		assert_eq!(mint_fee_for_point(9), total_fee / 10);
		// 1/(1/100)-1=99
		assert_eq!(mint_fee_for_point(99), total_fee / 100);
	});
}
