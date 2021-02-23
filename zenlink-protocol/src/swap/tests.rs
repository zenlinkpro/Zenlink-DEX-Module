// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use super::{mock::*, AssetId, Error};
use frame_support::{assert_noop, assert_ok};

const DOT: AssetId = AssetId::ParaCurrency(0);
const BTC: AssetId = AssetId::ParaCurrency(1);
const DEV: AssetId = AssetId::NativeCurrency;

const PAIR_ACCOUNT_0: u128 = 15310315390164549602772283245;

const ALICE: u128 = 1;
const BOB: u128 = 2;
const DOT_UNIT: u128 = 1000_000_000_000_000; //10^15
const BTC_UNIT: u128 = 1000_000_00; //10^8
const DEV_UNIT: u128 = 1000_000_000_000; //10^12

#[test]
fn inner_create_pair_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
		assert_ok!(DexModule::inner_create_pair(&BTC, &DEV));
		assert_ok!(DexModule::inner_create_pair(&DOT, &DEV));
	});
}

#[test]
fn inner_create_pair_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
		assert_noop!(
			DexModule::inner_create_pair(&BTC, &DOT),
			Error::<Test>::PairAlreadyExists
		);
	});
}

#[test]
fn inner_add_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexModule::inner_mint(DOT, &ALICE, u128::MAX));
		assert_ok!(DexModule::inner_mint(BTC, &ALICE, u128::MAX));
		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
		assert_ok!(DexModule::inner_create_pair(&BTC, &DEV));

		let total_supply_dot: u128 = 1 * DOT_UNIT;
		let total_supply_btc: u128 = 1 * BTC_UNIT;

		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));

		let total_supply_dot = 50 * DOT_UNIT;
		let total_supply_btc = 50 * BTC_UNIT;

		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE,
			&BTC,
			&DOT,
			total_supply_btc,
			total_supply_dot,
			0,
			0
		));

		let balance_dot = DexModule::balance_of(DOT, &PAIR_ACCOUNT_0);
		let balance_btc = DexModule::balance_of(BTC, &PAIR_ACCOUNT_0);
		println!("balance_DOT {}, balance_BTC {}", balance_dot, balance_btc);
		assert!((balance_dot / DOT_UNIT) == (balance_btc / BTC_UNIT));
	});
}

#[test]
fn inner_get_in_price_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexModule::inner_mint(DOT, &ALICE, u128::MAX));
		assert_ok!(DexModule::inner_mint(BTC, &ALICE, u128::MAX));
		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
		assert_ok!(DexModule::inner_create_pair(&BTC, &DEV));

		let total_supply_dot = 10000 * DOT_UNIT;
		let total_supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
		let path = vec![DOT.clone(), BTC.clone()];
		let amount_in = 1 * DOT_UNIT;

		let target_amount = DexModule::get_amount_out_by_path(amount_in, &path).unwrap();
		println!("target_amount {:#?}", target_amount);
		assert!(
			*target_amount.last().unwrap() < BTC_UNIT * 997 / 1000
				&& *target_amount.last().unwrap() > BTC_UNIT * 996 / 1000
		);

		let path = vec![BTC.clone(), DOT.clone()];
		let amount_in = 1 * BTC_UNIT;

		let target_amount = DexModule::get_amount_out_by_path(amount_in, &path).unwrap();
		println!("target_amount {:#?}", target_amount);
		assert!(
			*target_amount.last().unwrap() < DOT_UNIT * 997 / 1000
				&& *target_amount.last().unwrap() > DOT_UNIT * 996 / 1000
		);
	});
}

#[test]
fn inner_get_out_price_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexModule::inner_mint(DOT, &ALICE, u128::MAX));
		assert_ok!(DexModule::inner_mint(BTC, &ALICE, u128::MAX));
		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
		assert_ok!(DexModule::inner_create_pair(&BTC, &DEV));

		let total_supply_dot = 1000000 * DOT_UNIT;
		let total_supply_btc = 1000000 * BTC_UNIT;

		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
		let path = vec![DOT.clone(), BTC.clone()];
		let amount_out = 1 * BTC_UNIT;

		let target_amount = DexModule::get_amount_in_by_path(amount_out, &path).unwrap();
		println!("target_amount {:#?}", target_amount);
		assert!(
			*target_amount.first().unwrap() > DOT_UNIT * 1003 / 1000
				&& *target_amount.first().unwrap() < DOT_UNIT * 1004 / 1000
		);

		let path = vec![BTC.clone(), DOT.clone()];
		let amount_out = 1 * DOT_UNIT;
		let target_amount = DexModule::get_amount_in_by_path(amount_out, &path).unwrap();
		println!("target_amount {:#?}", target_amount);
		assert!(
			*target_amount.first().unwrap() > BTC_UNIT * 1003 / 1000
				&& *target_amount.first().unwrap() < BTC_UNIT * 1004 / 1000
		);
	});
}

#[test]
fn remove_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexModule::inner_mint(DOT, &ALICE, u128::MAX));
		assert_ok!(DexModule::inner_mint(BTC, &ALICE, u128::MAX));

		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));

		let total_supply_dot = 50 * DOT_UNIT;
		let total_supply_btc = 50 * BTC_UNIT;
		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));

		assert_ok!(DexModule::inner_remove_liquidity(
			&ALICE,
			&BTC,
			&DOT,
			1 * BTC_UNIT,
			0u128,
			0u128,
			&BOB
		));

		let balance_dot = DexModule::balance_of(DOT, &BOB);
		let balance_btc = DexModule::balance_of(BTC, &BOB);
		println!(
			"balance_dot {}, balance_btc {}",
			(balance_dot / balance_btc),
			balance_btc
		);
		assert!((balance_dot / balance_btc) / (DOT_UNIT / BTC_UNIT) == 1);
	})
}

#[test]
fn inner_swap_exact_tokens_for_tokens_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexModule::inner_mint(DOT, &ALICE, u128::MAX));
		assert_ok!(DexModule::inner_mint(BTC, &ALICE, u128::MAX));

		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));

		let total_supply_dot = 50000 * DOT_UNIT;
		let total_supply_btc = 50000 * BTC_UNIT;

		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
		let balance_dot = DexModule::balance_of(DOT, &PAIR_ACCOUNT_0);
		let balance_btc = DexModule::balance_of(BTC, &PAIR_ACCOUNT_0);
		println!("balance_dot:{} balance_btc{}", balance_dot, balance_btc);

		let path = vec![DOT.clone(), BTC.clone()];
		let amount_in = 1 * DOT_UNIT;
		let amount_out_min = BTC_UNIT * 996 / 1000;
		assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));

		let btc_balance = DexModule::balance_of(BTC, &BOB);
		println!("btc_balance {}", btc_balance);
		assert!(btc_balance > amount_out_min);

		let path = vec![BTC.clone(), DOT.clone()];
		let amount_in = 1 * BTC_UNIT;
		let amount_out_min = DOT_UNIT * 996 / 1000;
		assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));
		let dot_balance = DexModule::balance_of(DOT, &BOB);
		println!("dot_balance {}", dot_balance);
	})
}

#[test]
fn inner_swap_exact_tokens_for_tokens_in_pairs_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(DexModule::inner_mint(DOT, &ALICE, u128::MAX));
		assert_ok!(DexModule::inner_mint(BTC, &ALICE, u128::MAX));

		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
		assert_ok!(DexModule::inner_create_pair(&BTC, &DEV));

		let total_supply_dot = 5000 * DOT_UNIT;
		let total_supply_btc = 5000 * BTC_UNIT;
		let total_supply_dev = 5000 * DEV_UNIT;

		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE,
			&BTC,
			&DEV,
			total_supply_btc,
			total_supply_dev,
			0,
			0
		));

		let path = vec![DOT.clone(), BTC.clone(), DEV.clone()];
		let amount_in = 1 * DOT_UNIT;
		let amount_out_min = 1 * DEV_UNIT * 996 / 1000 * 996 / 1000;
		assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));
		let dev_balance = DexModule::asset_balance_of(&DEV, &BOB);
		println!("dot_balance {}", dev_balance);

		let path = vec![DEV.clone(), BTC.clone(), DOT.clone()];
		let amount_in = 1 * DEV_UNIT;
		let amount_out_min = 1 * DOT_UNIT * 996 / 1000 * 996 / 1000;
		let dot_balance = DexModule::asset_balance_of(&DOT, &BOB);
		println!("dot_balance {}", dot_balance);

		assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));
		let dot_balance = DexModule::asset_balance_of(&DOT, &BOB);
		println!("dot_balance {}", dot_balance);
	})
}

#[test]
fn inner_swap_tokens_for_exact_tokens_should_work() {
	new_test_ext().execute_with(|| {
		let total_supply_dot = 10000 * DOT_UNIT;
		let total_supply_btc = 10000 * BTC_UNIT;
		assert_ok!(DexModule::inner_mint(DOT, &ALICE, total_supply_dot));
		assert_ok!(DexModule::inner_mint(BTC, &ALICE, total_supply_btc));

		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));

		let supply_dot = 5000 * DOT_UNIT;
		let supply_btc = 5000 * BTC_UNIT;

		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE, &DOT, &BTC, supply_dot, supply_btc, 0, 0
		));
		let path = vec![DOT.clone(), BTC.clone()];
		let amount_out = 1 * BTC_UNIT;
		let amount_in_max = 1 * DOT_UNIT * 1004 / 1000;
		assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
		let btc_balance = DexModule::balance_of(BTC, &BOB);
		assert_eq!(btc_balance, amount_out);
		println!(
			"amount in {}",
			total_supply_dot - supply_dot - DexModule::balance_of(DOT, &ALICE)
		);
		assert!(total_supply_dot - supply_dot - DexModule::balance_of(DOT, &ALICE) < amount_in_max);

		let path = vec![BTC.clone(), DOT.clone()];
		let amount_out = 1 * DOT_UNIT;
		let amount_in_max = 1 * BTC_UNIT * 1004 / 1000;
		assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
		let dot_balance = DexModule::asset_balance_of(&DOT, &BOB);
		println!("dot_balance {}", dot_balance);
		assert_eq!(dot_balance, amount_out);
		println!(
			"amount in {}",
			total_supply_btc - supply_btc - DexModule::balance_of(BTC, &ALICE)
		);
		assert!(total_supply_btc - supply_btc - DexModule::balance_of(BTC, &ALICE) < amount_in_max);
	})
}

#[test]
fn inner_swap_tokens_for_exact_tokens_in_pairs_should_work() {
	new_test_ext().execute_with(|| {
		let total_supply_dot = 10000 * DOT_UNIT;
		let total_supply_btc = 10000 * BTC_UNIT;

		assert_ok!(DexModule::inner_mint(DOT, &ALICE, total_supply_dot));
		assert_ok!(DexModule::inner_mint(BTC, &ALICE, total_supply_btc));

		assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
		assert_ok!(DexModule::inner_create_pair(&BTC, &DEV));

		let supply_dot = 5000 * DOT_UNIT;
		let supply_btc = 5000 * BTC_UNIT;
		let supply_dev = 5000 * DEV_UNIT;

		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE, &DOT, &BTC, supply_dot, supply_btc, 0, 0
		));

		assert_ok!(DexModule::inner_add_liquidity(
			&ALICE, &BTC, &DEV, supply_btc, supply_dev, 0, 0
		));

		let path = vec![DOT.clone(), BTC.clone(), DEV.clone()];
		let amount_out = 1 * DEV_UNIT;
		let amount_in_max = 1 * DOT_UNIT * 1004 / 1000 * 1004 / 1000;
		let bob_dev_balance = DexModule::asset_balance_of(&DEV, &BOB);
		assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
		let dev_balance = DexModule::asset_balance_of(&DEV, &BOB);
		println!("dev_balance {}", dev_balance);
		assert_eq!(dev_balance - bob_dev_balance, amount_out);

		let path = vec![DEV.clone(), BTC.clone(), DOT.clone()];
		let amount_out = 1 * DOT_UNIT;
		let amount_in_max = 1 * DEV_UNIT * 1004 / 1000 * 1004 / 1000;
		assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
		let dot_balance = DexModule::asset_balance_of(&DOT, &BOB);
		assert_eq!(dot_balance, amount_out);
	})
}
