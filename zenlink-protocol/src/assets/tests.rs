// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use frame_support::{assert_noop, assert_ok};

use crate::{
    primitives::{AssetProperty, MultiAsset},
    NATIVE_CURRENCY_MODULE_INDEX,
};

pub const ASSET_MODULE_INDEX: u8 = 9;

use super::{mock::*, AssetId, Error};

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;

const ZLK_ASSET_ID: AssetId =
    AssetId { chain_id: 0, module_index: ASSET_MODULE_INDEX, asset_index: 0 };

const DEV_ASSET_ID: AssetId =
    AssetId { chain_id: 0, module_index: NATIVE_CURRENCY_MODULE_INDEX, asset_index: 0 };

const WDEV_ASSET_ID: AssetId =
    AssetId { chain_id: 0, module_index: ASSET_MODULE_INDEX, asset_index: 1 };

#[test]
fn inner_mint_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);
    });
}

#[test]
fn inner_mint_more_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_issue(WDEV_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);
        assert_ok!(Assets::inner_mint(WDEV_ASSET_ID, &BOB, 200));
        assert_eq!(Assets::balance_of(WDEV_ASSET_ID, &BOB), 200);
    });
}

#[test]
fn querying_assets_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_ok!(Assets::transfer(Origin::signed(ALICE), ZLK_ASSET_ID, BOB, 50));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 50);
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &BOB), 50);

        assert_eq!(Assets::total_supply(ZLK_ASSET_ID), 100);
    });
}

#[test]
fn querying_total_supply_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);
        assert_ok!(Assets::transfer(Origin::signed(ALICE), ZLK_ASSET_ID, BOB, 50));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 50);
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &BOB), 50);
        assert_ok!(Assets::transfer(Origin::signed(BOB), ZLK_ASSET_ID, CHARLIE, 31));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 50);
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &BOB), 19);
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &CHARLIE), 31);
        assert_eq!(Assets::total_supply(ZLK_ASSET_ID), 100);
    });
}

#[test]
fn transferring_amount_above_available_balance_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);
        assert_ok!(Assets::transfer(Origin::signed(ALICE), ZLK_ASSET_ID, 2, 50));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 50);
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &BOB), 50);
    });
}

#[test]
fn transferring_zero_unit_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &BOB), 0);

        assert_ok!(Assets::transfer(Origin::signed(ALICE), ZLK_ASSET_ID, BOB, 0));

        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &BOB), 0);
    });
}

#[test]
fn transferring_more_units_than_total_supply_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);
        assert_noop!(
            Assets::transfer(Origin::signed(ALICE), ZLK_ASSET_ID, BOB, 101),
            Error::<Test>::InsufficientAssetBalance
        );
    });
}

#[test]
fn inner_burn_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);

        assert_ok!(Assets::inner_burn(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 0);
    });
}

#[test]
fn inner_burn_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);

        assert_noop!(
            Assets::inner_burn(ZLK_ASSET_ID, &ALICE, 200),
            Error::<Test>::InsufficientAssetBalance,
        );
    });
}

#[test]
fn inner_mint_transfer_burn_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 100);
        assert_ok!(Assets::inner_burn(ZLK_ASSET_ID, &ALICE, 100));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 200));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 200);

        assert_ok!(Assets::transfer(Origin::signed(ALICE), ZLK_ASSET_ID, BOB, 150));
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &ALICE), 50);
        assert_eq!(Assets::balance_of(ZLK_ASSET_ID, &BOB), 150);

        assert_ok!(Assets::inner_burn(ZLK_ASSET_ID, &BOB, 150));

        assert_noop!(
            Assets::inner_burn(ZLK_ASSET_ID, &ALICE, 200),
            Error::<Test>::InsufficientAssetBalance,
        );
    });
}

#[test]
fn inner_multi_asset_total_supply_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));

        assert_eq!(Assets::multi_asset_total_supply(&DEV_ASSET_ID), CURRENCY_AMOUNT * 5);
        assert_eq!(Assets::multi_asset_total_supply(&ZLK_ASSET_ID), 100);
    });
}

#[test]
fn inner_multi_asset_withdraw_to_zenlink_module_should_work() {
    new_test_ext().execute_with(|| {
        let dev_total_supply = Assets::multi_asset_total_supply(&DEV_ASSET_ID);

        assert_ok!(Assets::multi_asset_withdraw(&DEV_ASSET_ID, &ALICE, 100));

        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_issue(WDEV_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::multi_asset_deposit(&WDEV_ASSET_ID, &ALICE, 100));

        assert_eq!(
            Assets::multi_asset_total_supply(&DEV_ASSET_ID)
                + Assets::multi_asset_balance_of(&WDEV_ASSET_ID, &ALICE),
            dev_total_supply
        );
    });
}

#[test]
fn inner_multi_asset_withdraw_and_deposit_should_work() {
    new_test_ext().execute_with(|| {
        let dev_total_supply = Assets::multi_asset_total_supply(&DEV_ASSET_ID);
        assert_ok!(Assets::multi_asset_withdraw(&DEV_ASSET_ID, &ALICE, 100));

        assert_ok!(Assets::multi_asset_deposit(&DEV_ASSET_ID, &ALICE, 100));

        assert_eq!(Assets::multi_asset_total_supply(&DEV_ASSET_ID), dev_total_supply);
    });
}

#[test]
fn inner_multi_asset_transfer_should_work() {
    new_test_ext().execute_with(|| {
        let balance_alice = Assets::multi_asset_balance_of(&DEV_ASSET_ID, &ALICE);
        assert_ok!(Assets::multi_asset_transfer(&DEV_ASSET_ID, &ALICE, &BOB, 200));
        assert_eq!(Assets::multi_asset_balance_of(&DEV_ASSET_ID, &ALICE), balance_alice - 200);

        assert_ok!(Assets::inner_issue(ZLK_ASSET_ID, AssetProperty::Foreign));
        assert_ok!(Assets::inner_mint(ZLK_ASSET_ID, &ALICE, 100));

        assert_ok!(Assets::multi_asset_transfer(&ZLK_ASSET_ID, &ALICE, &BOB, 50));
        assert_eq!(Assets::multi_asset_balance_of(&ZLK_ASSET_ID, &ALICE), 100 - 50);
    });
}
