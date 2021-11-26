// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use frame_support::{assert_noop, assert_ok};

use super::{mock::*, AssetId, Error, MultiAssetsHandler};
use crate::primitives::PairStatus::Trading;
use sp_runtime::DispatchError::BadOrigin;

const PAIR_DOT_BTC_ACCOUNT: u128 = 111825939709248857954450132390071529325;

const DOT_ASSET_ID: AssetId = AssetId {
	chain_id: 200,
	asset_type: LOCAL,
	asset_index: 2,
};

const BTC_ASSET_ID: AssetId = AssetId {
	chain_id: 300,
	asset_type: RESERVED,
	asset_index: 3,
};

const ETH_ASSET_ID: AssetId = AssetId {
	chain_id: 300,
	asset_type: NATIVE,
	asset_index: 0,
};

const DOT_BTC_LP_ID: AssetId = AssetId {
	chain_id: 0,
	asset_type: 2,
	asset_index: 12885034496,
};

const PAIR_DOT_BTC: u128 = 111825939709248857954450132390071529325;

const ALICE: u128 = 1;
const BOB: u128 = 2;
const DOT_UNIT: u128 = 1000_000_000_000_000;
const BTC_UNIT: u128 = 1000_000_00;
const ETH_UNIT: u128 = 1000_000_000_000;

#[test]
fn add_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));

		let total_supply_dot: u128 = 1 * DOT_UNIT;
		let total_supply_btc: u128 = 1 * BTC_UNIT;

		assert_ok!(DexPallet::add_liquidity(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0,
			100
		));

		let mint_liquidity = <Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &ALICE);

		assert_eq!(mint_liquidity, 316227766016);
		let total_supply_dot = 50 * DOT_UNIT;
		let total_supply_btc = 50 * BTC_UNIT;

		assert_ok!(DexPallet::add_liquidity(
			Origin::signed(ALICE),
			BTC_ASSET_ID,
			DOT_ASSET_ID,
			total_supply_btc,
			total_supply_dot,
			0,
			0,
			100
		));

		let balance_dot = <Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &PAIR_DOT_BTC);
		let balance_btc = <Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &PAIR_DOT_BTC);

		let mint_liquidity = <Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &ALICE);
		assert_eq!(mint_liquidity, 16127616066816);

		assert_eq!(balance_dot, 51000000000000000);
		assert_eq!(balance_btc, 5100000000);

		assert_eq!((balance_dot / DOT_UNIT), (balance_btc / BTC_UNIT));
	});
}

#[test]
fn remove_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));

		let total_supply_dot = 50 * DOT_UNIT;
		let total_supply_btc = 50 * BTC_UNIT;
		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));

		assert_ok!(DexPallet::remove_liquidity(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			1 * BTC_UNIT,
			0u128,
			0u128,
			BOB,
			100
		));

		let balance_dot = <Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &BOB);
		let balance_btc = <Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &BOB);

		assert_eq!(balance_dot, 316227766016);
		assert_eq!(balance_btc, 31622);

		assert_eq!((balance_dot / balance_btc) / (DOT_UNIT / BTC_UNIT), 1);
	})
}

#[test]
fn foreign_get_in_price_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));

		let total_supply_dot = 10000 * DOT_UNIT;
		let total_supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
		let path = vec![DOT_ASSET_ID, BTC_ASSET_ID];
		let amount_in = 1 * DOT_UNIT;

		let target_amount = DexPallet::get_amount_out_by_path(amount_in, &path).unwrap();

		assert_eq!(target_amount, vec![1000000000000000, 99690060]);

		assert!(
			*target_amount.last().unwrap() < BTC_UNIT * 997 / 1000
				&& *target_amount.last().unwrap() > BTC_UNIT * 996 / 1000
		);

		let path = vec![BTC_ASSET_ID, DOT_ASSET_ID];
		let amount_in = 1 * BTC_UNIT;

		let target_amount = DexPallet::get_amount_out_by_path(amount_in, &path).unwrap();

		assert_eq!(target_amount, vec![100000000, 996900609009281]);

		assert!(
			*target_amount.last().unwrap() < DOT_UNIT * 997 / 1000
				&& *target_amount.last().unwrap() > DOT_UNIT * 996 / 1000
		);
	});
}

#[test]
fn foreign_get_out_price_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));

		let total_supply_dot = 1000000 * DOT_UNIT;
		let total_supply_btc = 1000000 * BTC_UNIT;

		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
		let path = vec![DOT_ASSET_ID, BTC_ASSET_ID];
		let amount_out = 1 * BTC_UNIT;

		let target_amount = DexPallet::get_amount_in_by_path(amount_out, &path).unwrap();

		// println!("target_amount {:#?}", target_amount);
		assert_eq!(target_amount, vec![1003010030091274, 100000000]);

		assert!(
			*target_amount.first().unwrap() > DOT_UNIT * 1003 / 1000
				&& *target_amount.first().unwrap() < DOT_UNIT * 1004 / 1000
		);

		let path = vec![BTC_ASSET_ID, DOT_ASSET_ID];
		let amount_out = 1 * DOT_UNIT;
		let target_amount = DexPallet::get_amount_in_by_path(amount_out, &path).unwrap();

		// println!("target_amount {:#?}", target_amount);
		assert_eq!(target_amount, vec![100301004, 1000000000000000]);

		assert!(
			*target_amount.first().unwrap() > BTC_UNIT * 1003 / 1000
				&& *target_amount.first().unwrap() < BTC_UNIT * 1004 / 1000
		);
	});
}

#[test]
fn inner_swap_exact_assets_for_assets_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

		let total_supply_dot = 50000 * DOT_UNIT;
		let total_supply_btc = 50000 * BTC_UNIT;

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));

		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
		let balance_dot = <Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &PAIR_DOT_BTC);
		let balance_btc = <Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &PAIR_DOT_BTC);

		// println!("balance_dot {} balance_btc {}", balance_dot, balance_btc);
		assert_eq!(balance_dot, 50000000000000000000);
		assert_eq!(balance_btc, 5000000000000);

		let path = vec![DOT_ASSET_ID, BTC_ASSET_ID];
		let amount_in = 1 * DOT_UNIT;
		let amount_out_min = BTC_UNIT * 996 / 1000;
		assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));

		let btc_balance = <Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &BOB);

		// println!("btc_balance {}", btc_balance);
		assert_eq!(btc_balance, 99698012);

		assert!(btc_balance > amount_out_min);

		let path = vec![BTC_ASSET_ID.clone(), DOT_ASSET_ID.clone()];
		let amount_in = 1 * BTC_UNIT;
		let amount_out_min = DOT_UNIT * 996 / 1000;
		assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));
		let dot_balance = <Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &BOB);

		// println!("dot_balance {}", dot_balance);
		assert_eq!(dot_balance, 997019939603584)
	})
}

#[test]
fn inner_swap_exact_assets_for_assets_in_pairs_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(ETH_ASSET_ID, &ALICE, u128::MAX));

		let total_supply_dot = 5000 * DOT_UNIT;
		let total_supply_btc = 5000 * BTC_UNIT;
		let total_supply_eth = 5000 * ETH_UNIT;

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));

		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));

		assert_ok!(DexPallet::create_pair(Origin::root(), ETH_ASSET_ID, BTC_ASSET_ID,));

		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			BTC_ASSET_ID,
			ETH_ASSET_ID,
			total_supply_btc,
			total_supply_eth,
			0,
			0
		));

		let path = vec![DOT_ASSET_ID, BTC_ASSET_ID, ETH_ASSET_ID];
		let amount_in = 1 * DOT_UNIT;
		let amount_out_min = 1 * ETH_UNIT * 996 / 1000 * 996 / 1000;
		assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));
		let eth_balance = <Test as Config>::MultiAssetsHandler::balance_of(ETH_ASSET_ID, &BOB);

		// println!("eth_balance {}", eth_balance);
		assert_eq!(eth_balance, 993613333572);

		let path = vec![ETH_ASSET_ID, BTC_ASSET_ID, DOT_ASSET_ID];
		let amount_in = 1 * ETH_UNIT;
		let amount_out_min = 1 * DOT_UNIT * 996 / 1000 * 996 / 1000;
		let dot_balance = <Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &BOB);

		// println!("dot_balance {}", dot_balance);
		assert_eq!(dot_balance, 0);

		assert_ok!(DexPallet::inner_swap_exact_assets_for_assets(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));
		let dot_balance = <Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &BOB);

		// println!("dot_balance {}", dot_balance);
		assert_eq!(dot_balance, 994405843102918);
	})
}

#[test]
fn inner_swap_assets_for_exact_assets_should_work() {
	new_test_ext().execute_with(|| {
		let total_supply_dot = 10000 * DOT_UNIT;
		let total_supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, total_supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, total_supply_btc));

		let supply_dot = 5000 * DOT_UNIT;
		let supply_btc = 5000 * BTC_UNIT;

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));

		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			supply_dot,
			supply_btc,
			0,
			0
		));
		let path = vec![DOT_ASSET_ID, BTC_ASSET_ID];
		let amount_out = 1 * BTC_UNIT;
		let amount_in_max = 1 * DOT_UNIT * 1004 / 1000;
		assert_ok!(DexPallet::inner_swap_assets_for_exact_assets(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
		let btc_balance = <Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &BOB);
		assert_eq!(btc_balance, amount_out);

		let amount_in_dot =
			total_supply_dot - supply_dot - <Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &ALICE);

		// println!("amount in {}", amount_in_dot);
		assert_eq!(amount_in_dot, 1003209669015047);

		assert!(amount_in_dot < amount_in_max);

		let path = vec![BTC_ASSET_ID, DOT_ASSET_ID];
		let amount_out = 1 * DOT_UNIT;
		let amount_in_max = 1 * BTC_UNIT * 1004 / 1000;
		assert_ok!(DexPallet::inner_swap_assets_for_exact_assets(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
		let dot_balance = <Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &BOB);

		// println!("dot_balance {}", dot_balance);
		assert_eq!(dot_balance, 1000000000000000);

		assert_eq!(dot_balance, amount_out);

		let amount_in_btc =
			total_supply_btc - supply_btc - <Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &ALICE);

		// println!("amount in {}", amount_in_btc);
		assert_eq!(amount_in_btc, 100280779);

		assert!(amount_in_btc < amount_in_max);
	})
}

#[test]
fn inner_swap_assets_for_exact_assets_in_pairs_should_work() {
	new_test_ext().execute_with(|| {
		let total_supply_dot = 10000 * DOT_UNIT;
		let total_supply_btc = 10000 * BTC_UNIT;
		let total_supply_eth = 10000 * ETH_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, total_supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, total_supply_btc));
		assert_ok!(DexPallet::foreign_mint(ETH_ASSET_ID, &ALICE, total_supply_eth));

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));

		assert_ok!(DexPallet::create_pair(Origin::root(), ETH_ASSET_ID, BTC_ASSET_ID,));

		let supply_dot = 5000 * DOT_UNIT;
		let supply_btc = 5000 * BTC_UNIT;
		let supply_dev = 5000 * ETH_UNIT;

		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			supply_dot,
			supply_btc,
			0,
			0
		));

		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			BTC_ASSET_ID,
			ETH_ASSET_ID,
			supply_btc,
			supply_dev,
			0,
			0
		));

		let path = vec![DOT_ASSET_ID, BTC_ASSET_ID, ETH_ASSET_ID];
		let amount_out = 1 * ETH_UNIT;
		let amount_in_max = 1 * DOT_UNIT * 1004 / 1000 * 1004 / 1000;
		let bob_dev_balance = <Test as Config>::MultiAssetsHandler::balance_of(ETH_ASSET_ID, &BOB);
		assert_ok!(DexPallet::inner_swap_assets_for_exact_assets(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
		let eth_balance = <Test as Config>::MultiAssetsHandler::balance_of(ETH_ASSET_ID, &BOB);

		// println!("eth_balance {}", eth_balance);
		assert_eq!(eth_balance, 1000000000000);

		assert_eq!(eth_balance - bob_dev_balance, amount_out);

		let path = vec![ETH_ASSET_ID, BTC_ASSET_ID, DOT_ASSET_ID];
		let amount_out = 1 * DOT_UNIT;
		let amount_in_max = 1 * ETH_UNIT * 1004 / 1000 * 1004 / 1000;
		assert_ok!(DexPallet::inner_swap_assets_for_exact_assets(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
		let dot_balance = <Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &BOB);
		assert_eq!(dot_balance, amount_out);
	})
}

#[test]
fn create_bootstrap_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(<Test as Config>::MultiAssetsHandler::deposit(DOT_ASSET_ID, &ALICE, 0));
		assert_ok!(<Test as Config>::MultiAssetsHandler::deposit(ETH_ASSET_ID, &ALICE, 0));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			ETH_ASSET_ID,
			1000,
			1000,
			1000,
			1000,
			10000,
		));

		assert_noop!(
			DexPallet::create_pair(Origin::root(), ETH_ASSET_ID, DOT_ASSET_ID),
			Error::<Test>::PairAlreadyExists
		);

		assert_noop!(
			DexPallet::bootstrap_create(
				Origin::root(),
				DOT_ASSET_ID,
				ETH_ASSET_ID,
				1000,
				1000,
				1000,
				1000,
				10000,
			),
			Error::<Test>::PairAlreadyExists
		);
	})
}

#[test]
fn update_bootstrap_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(<Test as Config>::MultiAssetsHandler::deposit(DOT_ASSET_ID, &ALICE, 0));
		assert_ok!(<Test as Config>::MultiAssetsHandler::deposit(ETH_ASSET_ID, &ALICE, 0));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			ETH_ASSET_ID,
			1000,
			1000,
			1000,
			1000,
			10000,
		));

		assert_ok!(DexPallet::bootstrap_update(
			Origin::root(),
			DOT_ASSET_ID,
			ETH_ASSET_ID,
			10000,
			10000,
			10000,
			10000,
			100000,
		));

		assert_noop!(
			DexPallet::bootstrap_update(
				Origin::signed(BOB),
				DOT_ASSET_ID,
				ETH_ASSET_ID,
				10000,
				10000,
				10000,
				10000,
				100000,
			),
			BadOrigin
		);
	})
}

#[test]
fn bootstrap_contribute_should_work() {
	new_test_ext().execute_with(|| {
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, supply_btc));
		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			ETH_ASSET_ID,
			20 * DOT_UNIT,
			1 * BTC_UNIT,
			20 * DOT_UNIT,
			1 * BTC_UNIT,
			10000,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			ETH_ASSET_ID,
			DOT_UNIT,
			0,
			1000,
		));
		let pair = DexPallet::sort_asset_id(DOT_ASSET_ID, ETH_ASSET_ID);
		assert_eq!(DexPallet::bootstrap_personal_supply((pair, ALICE)), (DOT_UNIT, 0));
	})
}

#[test]
fn bootstrap_contribute_end_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, supply_btc));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &BOB, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			2,
		));

		assert_noop!(
			DexPallet::bootstrap_end(Origin::signed(ALICE), DOT_ASSET_ID, BTC_ASSET_ID),
			Error::<Test>::UnqualifiedBootstrap
		);

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		assert_noop!(
			DexPallet::bootstrap_end(Origin::signed(ALICE), DOT_ASSET_ID, BTC_ASSET_ID),
			Error::<Test>::UnqualifiedBootstrap
		);

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		System::set_block_number(3);
		assert_ok!(DexPallet::bootstrap_end(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID
		));
	})
}

#[test]
fn bootstrap_contribute_claim_reward_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, supply_btc));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &BOB, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			2,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		System::set_block_number(3);
		assert_ok!(DexPallet::bootstrap_end(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID
		));

		let total_supply = 2000000000000;

		assert_ok!(match DexPallet::pair_status((DOT_ASSET_ID, BTC_ASSET_ID)) {
			Trading(x) => {
				assert_eq!(x.pair_account, PAIR_DOT_BTC_ACCOUNT);
				assert_eq!(x.total_supply, total_supply);
				Ok(())
			}
			_ => Err(()),
		});

		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &ALICE),
			0
		);

		assert_ok!(DexPallet::bootstrap_claim(
			Origin::signed(ALICE),
			ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			1000,
		));
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &ALICE),
			total_supply / 2
		);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &PAIR_DOT_BTC_ACCOUNT),
			total_supply / 2
		);

		assert_noop!(
			DexPallet::bootstrap_claim(Origin::signed(ALICE), ALICE, DOT_ASSET_ID, BTC_ASSET_ID, 1000,),
			Error::<Test>::ZeroContribute
		);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &ALICE),
			total_supply / 2
		);

		assert_ok!(DexPallet::bootstrap_claim(
			Origin::signed(BOB),
			BOB,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			1000,
		));
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &BOB),
			total_supply / 2
		);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &PAIR_DOT_BTC_ACCOUNT),
			0
		);
	})
}

#[test]
fn refund_in_disable_bootstrap_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, supply_btc));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &BOB, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			2,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			0 * BTC_UNIT,
			1000,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		assert_noop!(
			DexPallet::bootstrap_claim(Origin::signed(BOB), BOB, DOT_ASSET_ID, BTC_ASSET_ID, 1000,),
			Error::<Test>::NotInBootstrap
		);

		System::set_block_number(3);

		assert_ok!(DexPallet::bootstrap_refund(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
		));
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &BOB),
			supply_dot
		);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &BOB),
			supply_btc
		);

		assert_noop!(
			DexPallet::bootstrap_refund(Origin::signed(BOB), DOT_ASSET_ID, BTC_ASSET_ID,),
			Error::<Test>::ZeroContribute
		);

		assert_noop!(
			DexPallet::bootstrap_refund(Origin::signed(BOB), DOT_ASSET_ID, BTC_ASSET_ID,),
			Error::<Test>::ZeroContribute
		);
	})
}

#[test]
fn disable_bootstrap_removed_after_all_refund_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			2,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		System::set_block_number(3);

		assert_ok!(DexPallet::bootstrap_refund(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
		));

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));
	})
}

#[test]
fn bootstrap_pair_deny_swap_should_work() {
	new_test_ext().execute_with(|| {
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			1,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		let path = vec![BTC_ASSET_ID, DOT_ASSET_ID];
		let amount_out = 1 * DOT_UNIT;
		let amount_in_max = 1 * ETH_UNIT * 1004 / 1000 * 1004 / 1000;
		assert_noop!(
			DexPallet::swap_assets_for_exact_assets(Origin::signed(ALICE), amount_out, amount_in_max, path, BOB, 1000,),
			Error::<Test>::InvalidPath
		);

		assert_noop!(
			DexPallet::add_liquidity(
				Origin::signed(ALICE),
				DOT_ASSET_ID,
				BTC_ASSET_ID,
				10 * DOT_UNIT,
				1 * BTC_UNIT,
				0,
				0,
				100,
			),
			Error::<Test>::PairNotExists
		);

		assert_noop!(
			DexPallet::remove_liquidity(
				Origin::signed(ALICE),
				DOT_ASSET_ID,
				BTC_ASSET_ID,
				1000,
				1 * DOT_UNIT,
				1 * BTC_UNIT,
				BOB,
				100,
			),
			Error::<Test>::PairNotExists
		);
	})
}

#[test]
fn refund_in_success_bootstrap_should_not_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, supply_btc));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &BOB, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			2,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		assert_noop!(
			DexPallet::bootstrap_claim(Origin::signed(BOB), BOB, DOT_ASSET_ID, BTC_ASSET_ID, 1000,),
			Error::<Test>::NotInBootstrap
		);

		assert_noop!(
			DexPallet::bootstrap_refund(Origin::signed(BOB), DOT_ASSET_ID, BTC_ASSET_ID,),
			Error::<Test>::DenyRefund
		);

		System::set_block_number(3);

		assert_noop!(
			DexPallet::bootstrap_refund(Origin::signed(BOB), DOT_ASSET_ID, BTC_ASSET_ID,),
			Error::<Test>::DenyRefund
		);
	})
}

#[test]
fn refund_in_ongoing_bootstrap_should_not_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, supply_btc));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &BOB, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			2,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			1 * BTC_UNIT,
			1000,
		));

		assert_noop!(
			DexPallet::bootstrap_refund(Origin::signed(BOB), DOT_ASSET_ID, BTC_ASSET_ID,),
			Error::<Test>::DenyRefund
		);
	})
}

#[test]
fn create_pair_in_disable_bootstrap_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, 1 * DOT_UNIT));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, 1 * BTC_UNIT));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &BOB, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			2,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			0 * BTC_UNIT,
			1000,
		));

		System::set_block_number(3);

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID));
		assert_ok!(DexPallet::add_liquidity(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			1 * DOT_UNIT,
			1 * BTC_UNIT,
			0,
			0,
			100
		));

		assert_ok!(DexPallet::bootstrap_refund(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
		));
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &BOB),
			supply_dot
		);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &BOB),
			supply_btc
		);

		assert_noop!(
			DexPallet::bootstrap_refund(Origin::signed(BOB), DOT_ASSET_ID, BTC_ASSET_ID,),
			Error::<Test>::ZeroContribute
		);

		assert_noop!(
			DexPallet::bootstrap_refund(Origin::signed(BOB), DOT_ASSET_ID, BTC_ASSET_ID,),
			Error::<Test>::ZeroContribute
		);

		let mint_liquidity = <Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &ALICE);
		assert_eq!(mint_liquidity, 316227766016);
	})
}

#[test]
fn create_bootstrap_in_disable_bootstrap() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, 1 * DOT_UNIT));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, 1 * BTC_UNIT));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &BOB, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			2,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			0 * BTC_UNIT,
			1000,
		));

		System::set_block_number(3);
		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			4,
		));

		assert_noop!(
			DexPallet::bootstrap_refund(Origin::signed(BOB), DOT_ASSET_ID, BTC_ASSET_ID,),
			Error::<Test>::DenyRefund
		);

		assert_noop!(
			DexPallet::bootstrap_end(Origin::signed(ALICE), DOT_ASSET_ID, BTC_ASSET_ID),
			Error::<Test>::UnqualifiedBootstrap
		);

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			10 * DOT_UNIT,
			2 * BTC_UNIT,
			1000,
		));

		System::set_block_number(5);
		assert_ok!(DexPallet::bootstrap_end(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID
		));
		assert_ok!(DexPallet::bootstrap_claim(
			Origin::signed(BOB),
			BOB,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			1000,
		));
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &BOB),
			2000000000000
		);
	})
}

#[test]
fn create_pair_in_ongoing_bootstrap_should_not_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let supply_dot = 10000 * DOT_UNIT;
		let supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, 1 * DOT_UNIT));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, 1 * BTC_UNIT));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, supply_dot));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &BOB, supply_btc));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			20 * DOT_UNIT,
			2 * BTC_UNIT,
			2,
		));
		assert_noop!(
			DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,),
			Error::<Test>::PairAlreadyExists
		);
	})
}

#[test]
fn liquidity_at_boundary_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID,));

		assert_ok!(DexPallet::add_liquidity(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			u128::MAX,
			u128::MAX,
			0,
			0,
			100
		));
		let mint_liquidity = <Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &ALICE);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &ALICE),
			0
		);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &ALICE),
			0
		);

		assert_eq!(mint_liquidity, u128::MAX);

		assert_ok!(DexPallet::remove_liquidity(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			u128::MAX,
			0,
			0,
			ALICE,
			100,
		));

		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &ALICE),
			u128::MAX
		);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &ALICE),
			u128::MAX
		);

		assert_ok!(DexPallet::add_liquidity(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			u128::MAX,
			u128::MAX,
			0,
			0,
			100
		));

		assert_ok!(DexPallet::remove_liquidity(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			u128::MAX / 2,
			0,
			0,
			ALICE,
			100,
		));

		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(DOT_ASSET_ID, &ALICE),
			u128::MAX / 2
		);
		assert_eq!(
			<Test as Config>::MultiAssetsHandler::balance_of(BTC_ASSET_ID, &ALICE),
			u128::MAX / 2
		);
	})
}

#[test]
fn liquidity_at_boundary_forbid_trade_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

		assert_ok!(DexPallet::create_pair(Origin::root(), DOT_ASSET_ID, BTC_ASSET_ID));

		assert_ok!(DexPallet::add_liquidity(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			u128::MAX,
			u128::MAX,
			0,
			0,
			100
		));

		let path = vec![DOT_ASSET_ID, BTC_ASSET_ID];
		let amount_out = 1 * DOT_UNIT;
		let amount_in_max = 2 * DOT_UNIT;

		assert_noop!(
			DexPallet::inner_swap_assets_for_exact_assets(&BOB, amount_out, amount_in_max, &path, &BOB),
			Error::<Test>::Overflow
		);

		let amount_in = 2 * DOT_UNIT;
		let amount_out_min = 1 * DOT_UNIT;

		assert_noop!(
			DexPallet::inner_swap_exact_assets_for_assets(&BOB, amount_in, amount_out_min, &path, &BOB),
			Error::<Test>::Overflow
		);
	})
}

#[test]
fn bootstrap_contribute_claim_at_boundary_should_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &ALICE, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(DOT_ASSET_ID, &BOB, u128::MAX));
		assert_ok!(DexPallet::foreign_mint(BTC_ASSET_ID, &BOB, u128::MAX));

		assert_ok!(DexPallet::bootstrap_create(
			Origin::root(),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			u128::MAX,
			u128::MAX,
			u128::MAX,
			u128::MAX,
			2,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			u128::MAX,
			0,
			1000,
		));

		assert_ok!(DexPallet::bootstrap_contribute(
			Origin::signed(BOB),
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			0,
			u128::MAX,
			1000,
		));

		System::set_block_number(3);
		assert_ok!(DexPallet::bootstrap_end(
			Origin::signed(ALICE),
			DOT_ASSET_ID,
			BTC_ASSET_ID
		));

		assert_ok!(match DexPallet::pair_status((DOT_ASSET_ID, BTC_ASSET_ID)) {
			Trading(x) => {
				assert_eq!(x.pair_account, PAIR_DOT_BTC_ACCOUNT);
				assert_eq!(x.total_supply, u128::MAX);
				Ok(())
			}
			_ => Err(()),
		});

		assert_ok!(DexPallet::bootstrap_claim(
			Origin::signed(ALICE),
			ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			1000,
		));
		let alice_lp_token_amount = <Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &ALICE);
		assert_eq!(alice_lp_token_amount, u128::MAX / 2);

		assert_ok!(DexPallet::bootstrap_claim(
			Origin::signed(BOB),
			BOB,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			1000,
		));

		let bob_lp_token_amount = <Test as Config>::MultiAssetsHandler::balance_of(DOT_BTC_LP_ID, &BOB);
		assert_eq!(bob_lp_token_amount, u128::MAX / 2);
	})
}
