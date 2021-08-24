// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use frame_support::assert_ok;

use super::{mock::*, AssetId, MultiAssetsHandler};

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

		assert_ok!(DexPallet::inner_add_liquidity(
			&ALICE,
			DOT_ASSET_ID,
			BTC_ASSET_ID,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
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
