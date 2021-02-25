// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use frame_support::{assert_noop, assert_ok};

use super::{mock::*, AssetId, Error};

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;
const DOT: AssetId = AssetId::ParaCurrency(0);
const BTC: AssetId = AssetId::ParaCurrency(1);

#[test]
fn inner_mint_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);
    });
}

#[test]
fn inner_mint_more_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);
        assert_ok!(Assets::inner_mint(BTC, &BOB, 200));
        assert_eq!(Assets::balance_of(BTC, &BOB), 200);
    });
}

#[test]
fn querying_assets_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_ok!(Assets::inner_mint(BTC, &BOB, 200));
        assert_ok!(Assets::transfer(Origin::signed(ALICE), DOT, BOB, 50));
        assert_ok!(Assets::transfer(Origin::signed(BOB), BTC, ALICE, 40));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 50);
        assert_eq!(Assets::balance_of(DOT, &BOB), 50);
        assert_eq!(Assets::balance_of(BTC, &ALICE), 40);
        assert_eq!(Assets::balance_of(BTC, &BOB), 160);

        let assets = Assets::assets_list();
        let mut iter = assets.iter();
        assert_eq!(iter.next(), Some(&DOT));
        assert_eq!(iter.next(), Some(&BTC));
        assert_eq!(iter.next(), None);

        assert_eq!(Assets::total_supply(DOT), 100);
    });
}

#[test]
fn querying_total_supply_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);
        assert_ok!(Assets::transfer(Origin::signed(ALICE), DOT, BOB, 50));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 50);
        assert_eq!(Assets::balance_of(DOT, &BOB), 50);
        assert_ok!(Assets::transfer(Origin::signed(BOB), DOT, CHARLIE, 31));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 50);
        assert_eq!(Assets::balance_of(DOT, &BOB), 19);
        assert_eq!(Assets::balance_of(DOT, &CHARLIE), 31);
        assert_eq!(Assets::total_supply(DOT), 100);
    });
}

#[test]
fn transferring_amount_above_available_balance_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);
        assert_ok!(Assets::transfer(Origin::signed(ALICE), DOT, 2, 50));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 50);
        assert_eq!(Assets::balance_of(DOT, &BOB), 50);
    });
}

#[test]
fn transferring_zero_unit_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);
        assert_eq!(Assets::balance_of(DOT, &BOB), 0);

        assert_ok!(Assets::transfer(Origin::signed(ALICE), DOT, BOB, 0));

        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);
        assert_eq!(Assets::balance_of(DOT, &BOB), 0);
    });
}

#[test]
fn transferring_more_units_than_total_supply_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);
        assert_noop!(
            Assets::transfer(Origin::signed(ALICE), DOT, BOB, 101),
            Error::<Test>::InsufficientAssetBalance
        );
    });
}

#[test]
fn inner_burn_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);

        assert_ok!(Assets::inner_burn(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 0);
    });
}

#[test]
fn inner_burn_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);

        assert_noop!(
            Assets::inner_burn(DOT, &ALICE, 200),
            Error::<Test>::InsufficientAssetBalance,
        );
    });
}

#[test]
fn inner_mint_transfer_burn_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 100));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 100);
        assert_ok!(Assets::inner_burn(DOT, &ALICE, 100));
        assert_ok!(Assets::inner_mint(DOT, &ALICE, 200));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 200);

        assert_ok!(Assets::transfer(Origin::signed(ALICE), DOT, BOB, 150));
        assert_eq!(Assets::balance_of(DOT, &ALICE), 50);
        assert_eq!(Assets::balance_of(DOT, &BOB), 150);

        assert_ok!(Assets::inner_burn(DOT, &BOB, 150));

        assert_noop!(
            Assets::inner_burn(DOT, &ALICE, 200),
            Error::<Test>::InsufficientAssetBalance,
        );
    });
}
