// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use frame_support::{assert_noop, assert_ok};

pub const ASSET_MODULE_INDEX: u8 = 8;

use super::{mock::*, AssetId, AssetProperty, Error, MultiAssetHandler};

const DOT_ASSET_ID: AssetId =
    AssetId { chain_id: 300, module_index: ASSET_MODULE_INDEX, asset_index: 0 };

const BTC_ASSET_ID: AssetId =
    AssetId { chain_id: 300, module_index: ASSET_MODULE_INDEX, asset_index: 1 };

const DEV_ASSET_ID: AssetId =
    AssetId { chain_id: 300, module_index: ASSET_MODULE_INDEX, asset_index: 2 };

const PAIR_ACCOUNT_0: u128 = 15310315390164549602772283245;

const ALICE: u128 = 1;
const BOB: u128 = 2;
const DOT_UNIT: u128 = 1000_000_000_000_000;
const BTC_UNIT: u128 = 1000_000_00;
const DEV_UNIT: u128 = 1000_000_000_000;

#[test]
fn inner_create_pair_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(DEV_ASSET_ID, AssetProperty::Foreign));

        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));
        assert_ok!(DexModule::inner_create_pair(&BTC_ASSET_ID, &DEV_ASSET_ID));
        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &DEV_ASSET_ID));
    });
}

#[test]
fn inner_create_pair_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));
        assert_noop!(
            DexModule::inner_create_pair(&BTC_ASSET_ID, &DOT_ASSET_ID),
            Error::<Test>::PairAlreadyExists
        );
    });
}

#[test]
fn inner_add_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(DEV_ASSET_ID, AssetProperty::Foreign));

        assert_ok!(DexModule::inner_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_mint(BTC_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));
        assert_ok!(DexModule::inner_create_pair(&BTC_ASSET_ID, &DEV_ASSET_ID));

        let total_supply_dot: u128 = 1 * DOT_UNIT;
        let total_supply_btc: u128 = 1 * BTC_UNIT;

        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &DOT_ASSET_ID,
            &BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0
        ));

        let total_supply_dot = 50 * DOT_UNIT;
        let total_supply_btc = 50 * BTC_UNIT;

        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &BTC_ASSET_ID,
            &DOT_ASSET_ID,
            total_supply_btc,
            total_supply_dot,
            0,
            0
        ));

        let balance_dot = DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &PAIR_ACCOUNT_0);
        let balance_btc = DexModule::multi_asset_balance_of(&BTC_ASSET_ID, &PAIR_ACCOUNT_0);
        println!("balance_DOT {}, balance_BTC {}", balance_dot, balance_btc);
        assert_eq!((balance_dot / DOT_UNIT), (balance_btc / BTC_UNIT));
    });
}

#[test]
fn inner_get_in_price_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_mint(BTC_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));
        assert_ok!(DexModule::inner_create_pair(&BTC_ASSET_ID, &DEV_ASSET_ID));

        let total_supply_dot = 10000 * DOT_UNIT;
        let total_supply_btc = 10000 * BTC_UNIT;

        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &DOT_ASSET_ID,
            &BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0
        ));
        let path = vec![DOT_ASSET_ID.clone(), BTC_ASSET_ID.clone()];
        let amount_in = 1 * DOT_UNIT;

        let target_amount = DexModule::get_amount_out_by_path(amount_in, &path).unwrap();
        println!("target_amount {:#?}", target_amount);
        assert!(
            *target_amount.last().unwrap() < BTC_UNIT * 997 / 1000
                && *target_amount.last().unwrap() > BTC_UNIT * 996 / 1000
        );

        let path = vec![BTC_ASSET_ID.clone(), DOT_ASSET_ID.clone()];
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
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_mint(BTC_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));
        assert_ok!(DexModule::inner_create_pair(&BTC_ASSET_ID, &DEV_ASSET_ID));

        let total_supply_dot = 1000000 * DOT_UNIT;
        let total_supply_btc = 1000000 * BTC_UNIT;

        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &DOT_ASSET_ID,
            &BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0
        ));
        let path = vec![DOT_ASSET_ID.clone(), BTC_ASSET_ID.clone()];
        let amount_out = 1 * BTC_UNIT;

        let target_amount = DexModule::get_amount_in_by_path(amount_out, &path).unwrap();
        println!("target_amount {:#?}", target_amount);
        assert!(
            *target_amount.first().unwrap() > DOT_UNIT * 1003 / 1000
                && *target_amount.first().unwrap() < DOT_UNIT * 1004 / 1000
        );

        let path = vec![BTC_ASSET_ID.clone(), DOT_ASSET_ID.clone()];
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
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));

        let total_supply_dot = 50 * DOT_UNIT;
        let total_supply_btc = 50 * BTC_UNIT;
        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &DOT_ASSET_ID,
            &BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0
        ));

        assert_ok!(DexModule::inner_remove_liquidity(
            &ALICE,
            &DOT_ASSET_ID,
            &BTC_ASSET_ID,
            1 * BTC_UNIT,
            0u128,
            0u128,
            &BOB
        ));

        let balance_dot = DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &BOB);
        let balance_btc = DexModule::multi_asset_balance_of(&BTC_ASSET_ID, &BOB);
        println!("balance_dot {}, balance_btc {}", (balance_dot / balance_btc), balance_btc);
        assert_eq!((balance_dot / balance_btc) / (DOT_UNIT / BTC_UNIT), 1);
    })
}

#[test]
fn inner_swap_exact_tokens_for_tokens_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_mint(BTC_ASSET_ID, &ALICE, u128::MAX));

        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));

        let total_supply_dot = 50000 * DOT_UNIT;
        let total_supply_btc = 50000 * BTC_UNIT;

        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &DOT_ASSET_ID,
            &BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0
        ));
        let balance_dot = DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &PAIR_ACCOUNT_0);
        let balance_btc = DexModule::multi_asset_balance_of(&BTC_ASSET_ID, &PAIR_ACCOUNT_0);
        println!("balance_dot:{} balance_btc{}", balance_dot, balance_btc);

        let path = vec![DOT_ASSET_ID.clone(), BTC_ASSET_ID.clone()];
        let amount_in = 1 * DOT_UNIT;
        let amount_out_min = BTC_UNIT * 996 / 1000;
        assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens(
            &ALICE,
            amount_in,
            amount_out_min,
            &path,
            &BOB,
        ));

        let btc_balance = DexModule::multi_asset_balance_of(&BTC_ASSET_ID, &BOB);
        println!("btc_balance {}", btc_balance);
        assert!(btc_balance > amount_out_min);

        let path = vec![BTC_ASSET_ID.clone(), DOT_ASSET_ID.clone()];
        let amount_in = 1 * BTC_UNIT;
        let amount_out_min = DOT_UNIT * 996 / 1000;
        assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens(
            &ALICE,
            amount_in,
            amount_out_min,
            &path,
            &BOB,
        ));
        let dot_balance = DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &BOB);
        println!("dot_balance {}", dot_balance);
    })
}

#[test]
fn inner_swap_exact_tokens_for_tokens_in_pairs_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(DEV_ASSET_ID, AssetProperty::Foreign));

        assert_ok!(DexModule::inner_mint(DOT_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_mint(BTC_ASSET_ID, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_mint(DEV_ASSET_ID, &ALICE, u128::MAX));

        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));
        assert_ok!(DexModule::inner_create_pair(&BTC_ASSET_ID, &DEV_ASSET_ID));

        let total_supply_dot = 5000 * DOT_UNIT;
        let total_supply_btc = 5000 * BTC_UNIT;
        let total_supply_dev = 5000 * DEV_UNIT;

        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &DOT_ASSET_ID,
            &BTC_ASSET_ID,
            total_supply_dot,
            total_supply_btc,
            0,
            0
        ));
        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &BTC_ASSET_ID,
            &DEV_ASSET_ID,
            total_supply_btc,
            total_supply_dev,
            0,
            0
        ));

        let path = vec![DOT_ASSET_ID.clone(), BTC_ASSET_ID.clone(), DEV_ASSET_ID.clone()];
        let amount_in = 1 * DOT_UNIT;
        let amount_out_min = 1 * DEV_UNIT * 996 / 1000 * 996 / 1000;
        assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens(
            &ALICE,
            amount_in,
            amount_out_min,
            &path,
            &BOB,
        ));
        let dev_balance = DexModule::multi_asset_balance_of(&DEV_ASSET_ID, &BOB);
        println!("dot_balance {}", dev_balance);

        let path = vec![DEV_ASSET_ID.clone(), BTC_ASSET_ID.clone(), DOT_ASSET_ID.clone()];
        let amount_in = 1 * DEV_UNIT;
        let amount_out_min = 1 * DOT_UNIT * 996 / 1000 * 996 / 1000;
        let dot_balance = DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &BOB);
        println!("dot_balance {}", dot_balance);

        assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens(
            &ALICE,
            amount_in,
            amount_out_min,
            &path,
            &BOB,
        ));
        let dot_balance = DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &BOB);
        println!("dot_balance {}", dot_balance);
    })
}

#[test]
fn inner_swap_tokens_for_exact_tokens_should_work() {
    new_test_ext().execute_with(|| {
        let total_supply_dot = 10000 * DOT_UNIT;
        let total_supply_btc = 10000 * BTC_UNIT;
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_mint(DOT_ASSET_ID, &ALICE, total_supply_dot));
        assert_ok!(DexModule::inner_mint(BTC_ASSET_ID, &ALICE, total_supply_btc));

        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));

        let supply_dot = 5000 * DOT_UNIT;
        let supply_btc = 5000 * BTC_UNIT;

        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &DOT_ASSET_ID,
            &BTC_ASSET_ID,
            supply_dot,
            supply_btc,
            0,
            0
        ));
        let path = vec![DOT_ASSET_ID.clone(), BTC_ASSET_ID.clone()];
        let amount_out = 1 * BTC_UNIT;
        let amount_in_max = 1 * DOT_UNIT * 1004 / 1000;
        assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens(
            &ALICE,
            amount_out,
            amount_in_max,
            &path,
            &BOB
        ));
        let btc_balance = DexModule::multi_asset_balance_of(&BTC_ASSET_ID, &BOB);
        assert_eq!(btc_balance, amount_out);
        println!(
            "amount in {}",
            total_supply_dot
                - supply_dot
                - DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &ALICE)
        );
        assert!(
            total_supply_dot
                - supply_dot
                - DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &ALICE)
                < amount_in_max
        );

        let path = vec![BTC_ASSET_ID.clone(), DOT_ASSET_ID.clone()];
        let amount_out = 1 * DOT_UNIT;
        let amount_in_max = 1 * BTC_UNIT * 1004 / 1000;
        assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens(
            &ALICE,
            amount_out,
            amount_in_max,
            &path,
            &BOB
        ));
        let dot_balance = DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &BOB);
        println!("dot_balance {}", dot_balance);
        assert_eq!(dot_balance, amount_out);
        println!(
            "amount in {}",
            total_supply_btc
                - supply_btc
                - DexModule::multi_asset_balance_of(&BTC_ASSET_ID, &ALICE)
        );
        assert!(
            total_supply_btc
                - supply_btc
                - DexModule::multi_asset_balance_of(&BTC_ASSET_ID, &ALICE)
                < amount_in_max
        );
    })
}

#[test]
fn inner_swap_tokens_for_exact_tokens_in_pairs_should_work() {
    new_test_ext().execute_with(|| {
        let total_supply_dot = 10000 * DOT_UNIT;
        let total_supply_btc = 10000 * BTC_UNIT;
        let total_supply_dev = 10000 * DEV_UNIT;
        assert_ok!(DexModule::inner_issue(DOT_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(BTC_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_issue(DEV_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(DexModule::inner_mint(DOT_ASSET_ID, &ALICE, total_supply_dot));
        assert_ok!(DexModule::inner_mint(BTC_ASSET_ID, &ALICE, total_supply_btc));
        assert_ok!(DexModule::inner_mint(DEV_ASSET_ID, &ALICE, total_supply_dev));

        assert_ok!(DexModule::inner_create_pair(&DOT_ASSET_ID, &BTC_ASSET_ID));
        assert_ok!(DexModule::inner_create_pair(&BTC_ASSET_ID, &DEV_ASSET_ID));

        let supply_dot = 5000 * DOT_UNIT;
        let supply_btc = 5000 * BTC_UNIT;
        let supply_dev = 5000 * DEV_UNIT;

        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &DOT_ASSET_ID,
            &BTC_ASSET_ID,
            supply_dot,
            supply_btc,
            0,
            0
        ));

        assert_ok!(DexModule::inner_add_liquidity(
            &ALICE,
            &BTC_ASSET_ID,
            &DEV_ASSET_ID,
            supply_btc,
            supply_dev,
            0,
            0
        ));

        let path = vec![DOT_ASSET_ID.clone(), BTC_ASSET_ID.clone(), DEV_ASSET_ID.clone()];
        let amount_out = 1 * DEV_UNIT;
        let amount_in_max = 1 * DOT_UNIT * 1004 / 1000 * 1004 / 1000;
        let bob_dev_balance = DexModule::multi_asset_balance_of(&DEV_ASSET_ID, &BOB);
        assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens(
            &ALICE,
            amount_out,
            amount_in_max,
            &path,
            &BOB
        ));
        let dev_balance = DexModule::multi_asset_balance_of(&DEV_ASSET_ID, &BOB);
        println!("dev_balance {}", dev_balance);
        assert_eq!(dev_balance - bob_dev_balance, amount_out);

        let path = vec![DEV_ASSET_ID.clone(), BTC_ASSET_ID.clone(), DOT_ASSET_ID.clone()];
        let amount_out = 1 * DOT_UNIT;
        let amount_in_max = 1 * DEV_UNIT * 1004 / 1000 * 1004 / 1000;
        assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens(
            &ALICE,
            amount_out,
            amount_in_max,
            &path,
            &BOB
        ));
        let dot_balance = DexModule::multi_asset_balance_of(&DOT_ASSET_ID, &BOB);
        assert_eq!(dot_balance, amount_out);
    })
}
