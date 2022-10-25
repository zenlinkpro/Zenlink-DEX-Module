// Copyright 2021-2022 Zenlink.
// Licensed under Apache 2.0.

use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError::BadOrigin;

use super::*;
use crate::{
	mock::{CurrencyId::*, *},
	Error,
};

const ASSET1: CurrencyId = Token(TOKEN1_SYMBOL);
// const ASSET2: CurrencyId = Token(TOKEN2_SYMBOL);

const VAULT_ASSET1: CurrencyId = VaultToken(TOKEN1_SYMBOL);

const VAULT_PALLET_ACCOUNT: AccountId = 36018076922747156782236594029;

#[test]
fn basic_create_vault_asset_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(VaultPallet::create_vault_asset(
			Origin::root(),
			ASSET1,
			TOKEN1_DECIMAL,
			12,
			5e17 as Balance,
			0,
		));

		assert_eq!(VaultPallet::vault_asset(ASSET1), Some(VAULT_ASSET1));
		assert_eq!(
			VaultPallet::asset_ratio(ASSET1),
			Some(Ratio { max_penalty_ratio: 5e17 as Balance, min_penalty_ratio: 0 })
		);
		assert_eq!(
			VaultPallet::asset_metadata(ASSET1),
			Some(Metadata { decimal: TOKEN1_DECIMAL, related_asset_id: VAULT_ASSET1 })
		);
		assert_eq!(
			VaultPallet::asset_metadata(VAULT_ASSET1),
			Some(Metadata { decimal: 12, related_asset_id: ASSET1 })
		);
	})
}

#[test]
fn create_vault_asset_with_non_admin_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			VaultPallet::create_vault_asset(
				Origin::signed(ALICE),
				ASSET1,
				TOKEN1_DECIMAL,
				TOKEN1_DECIMAL,
				5e17 as Balance,
				0,
			),
			BadOrigin
		);
	})
}

#[test]
fn create_vault_asset_repeatedly_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(VaultPallet::create_vault_asset(
			Origin::root(),
			ASSET1,
			TOKEN1_DECIMAL,
			TOKEN1_DECIMAL,
			5e17 as Balance,
			0,
		));

		assert_noop!(
			VaultPallet::create_vault_asset(
				Origin::root(),
				ASSET1,
				TOKEN1_DECIMAL,
				TOKEN2_DECIMAL,
				4e17 as Balance,
				2e17 as Balance,
			),
			Error::<Test>::VaultAssetExisted
		);

		// nothing changed
		assert_eq!(VaultPallet::vault_asset(ASSET1), Some(VAULT_ASSET1));
		assert_eq!(
			VaultPallet::asset_ratio(ASSET1),
			Some(Ratio { max_penalty_ratio: 5e17 as Balance, min_penalty_ratio: 0 })
		);
		assert_eq!(
			VaultPallet::asset_metadata(ASSET1),
			Some(Metadata { decimal: TOKEN1_DECIMAL, related_asset_id: VAULT_ASSET1 })
		);
		assert_eq!(
			VaultPallet::asset_metadata(VAULT_ASSET1),
			Some(Metadata { decimal: TOKEN1_DECIMAL, related_asset_id: ASSET1 })
		);
	})
}

#[test]
fn first_deposit_with_empty_vault_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(VaultPallet::create_vault_asset(
			Origin::root(),
			ASSET1,
			TOKEN1_DECIMAL,
			TOKEN1_DECIMAL,
			5e17 as Balance,
			0,
		));

		assert_eq!(VaultPallet::max_deposit(ASSET1, &ALICE), Ok(Balance::MAX));
		assert_eq!(VaultPallet::preview_deposit(ASSET1, 1e18 as Balance), Ok(1e18 as Balance));

		let alice_asset1_balance_before = get_user_balance(ASSET1, &ALICE);

		assert_ok!(VaultPallet::deposit(Origin::signed(ALICE), ASSET1, 1e18 as Balance, ALICE,));

		let alice_asset1_balance_after = get_user_balance(ASSET1, &ALICE);

		assert_eq!(alice_asset1_balance_before - alice_asset1_balance_after, 1e18 as Balance);
		assert_eq!(get_user_balance(VAULT_ASSET1, &ALICE), 1e18 as Balance);
	})
}

#[test]
fn first_mint_with_empty_vault_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(VaultPallet::create_vault_asset(
			Origin::root(),
			ASSET1,
			TOKEN1_DECIMAL,
			TOKEN1_DECIMAL,
			5e17 as Balance,
			0,
		));

		assert_eq!(VaultPallet::max_mint(ASSET1, &ALICE), Ok(Balance::MAX));
		assert_eq!(VaultPallet::preview_mint(ASSET1, 1e18 as Balance), Ok(1e18 as Balance));

		let alice_asset1_balance_before = get_user_balance(ASSET1, &ALICE);

		assert_ok!(VaultPallet::mint(Origin::signed(ALICE), ASSET1, 1e18 as Balance, ALICE,));

		let alice_asset1_balance_after = get_user_balance(ASSET1, &ALICE);

		assert_eq!(alice_asset1_balance_before - alice_asset1_balance_after, 1e18 as Balance);
		assert_eq!(get_user_balance(VAULT_ASSET1, &ALICE), 1e18 as Balance);
	})
}

#[test]
fn first_withdraw_with_empty_vault_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(VaultPallet::create_vault_asset(
			Origin::root(),
			ASSET1,
			TOKEN1_DECIMAL,
			TOKEN1_DECIMAL,
			5e17 as Balance,
			0,
		));

		assert_eq!(VaultPallet::max_withdraw(ASSET1, &ALICE), Ok(0));
		assert_eq!(VaultPallet::preview_withdraw(ASSET1, 0), Ok(0));

		let alice_asset1_balance_before = get_user_balance(ASSET1, &ALICE);

		assert_ok!(VaultPallet::withdraw(Origin::signed(ALICE), ASSET1, 0, ALICE,));

		let alice_asset1_balance_after = get_user_balance(ASSET1, &ALICE);

		assert_eq!(alice_asset1_balance_before, alice_asset1_balance_after);
		assert_eq!(get_user_balance(VAULT_ASSET1, &ALICE), 0);
	})
}

#[test]
fn first_redeem_with_empty_vault_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(VaultPallet::create_vault_asset(
			Origin::root(),
			ASSET1,
			TOKEN1_DECIMAL,
			TOKEN1_DECIMAL,
			5e17 as Balance,
			0,
		));

		assert_eq!(VaultPallet::max_redeem(ASSET1, &ALICE), Ok(0));
		assert_eq!(VaultPallet::preview_redeem(ASSET1, 0), Ok(0));

		let alice_asset1_balance_before = get_user_balance(ASSET1, &ALICE);

		assert_ok!(VaultPallet::redeem(Origin::signed(ALICE), ASSET1, 0, ALICE,));

		let alice_asset1_balance_after = get_user_balance(ASSET1, &ALICE);

		assert_eq!(alice_asset1_balance_before, alice_asset1_balance_after);
		assert_eq!(get_user_balance(VAULT_ASSET1, &ALICE), 0);
	})
}

fn prepare_vault_with_asset_but_no_share(underlying_asset_id: CurrencyId) -> CurrencyId {
	assert_ok!(VaultPallet::create_vault_asset(
		Origin::root(),
		underlying_asset_id,
		TOKEN1_DECIMAL,
		TOKEN1_DECIMAL,
		5e17 as Balance,
		0,
	));

	transfer_from(underlying_asset_id, &ALICE, 1e18 as Balance, &VAULT_PALLET_ACCOUNT);

	VaultPallet::vault_asset(underlying_asset_id).unwrap()
}

#[test]
fn vault_with_assets_but_no_share_should_work() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		prepare_vault_with_asset_but_no_share(underlying_asset_id);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(1e18 as Balance));
	})
}

#[test]
fn deposit_vault_with_assets_but_no_share_should_work() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		let vault_asset_id = prepare_vault_with_asset_but_no_share(underlying_asset_id);
		assert_eq!(VaultPallet::max_deposit(underlying_asset_id, &ALICE), Ok(Balance::MAX));
		assert_eq!(
			VaultPallet::preview_deposit(underlying_asset_id, 1e18 as Balance),
			Ok(1e18 as Balance)
		);

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_ok!(VaultPallet::deposit(Origin::signed(ALICE), ASSET1, 1e18 as Balance, ALICE,));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);

		assert_eq!(underlying_balance_before - underlying_balance_after, 1e18 as Balance);
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 1e18 as Balance);
	})
}

#[test]
fn mint_vault_with_assets_but_no_share_should_work() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		let vault_asset_id = prepare_vault_with_asset_but_no_share(underlying_asset_id);
		assert_eq!(VaultPallet::max_mint(underlying_asset_id, &ALICE), Ok(Balance::MAX));
		assert_eq!(
			VaultPallet::preview_mint(underlying_asset_id, 1e18 as Balance),
			Ok(1e18 as Balance)
		);

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_ok!(VaultPallet::mint(Origin::signed(ALICE), ASSET1, 1e18 as Balance, ALICE,));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);

		assert_eq!(underlying_balance_before - underlying_balance_after, 1e18 as Balance);
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 1e18 as Balance);
	})
}

#[test]
fn withdraw_vault_with_assets_but_no_share_should_work() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		let vault_asset_id = prepare_vault_with_asset_but_no_share(underlying_asset_id);
		assert_eq!(VaultPallet::max_withdraw(underlying_asset_id, &ALICE), Ok(0));
		assert_eq!(
			VaultPallet::preview_mint(underlying_asset_id, 1e18 as Balance),
			Ok(1e18 as Balance)
		);

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_ok!(VaultPallet::withdraw(Origin::signed(ALICE), ASSET1, 0, ALICE,));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);

		assert_eq!(underlying_balance_before, underlying_balance_after);
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 0);
	})
}

#[test]
fn redeem_vault_with_assets_but_no_share_should_work() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		let vault_asset_id = prepare_vault_with_asset_but_no_share(underlying_asset_id);
		assert_eq!(VaultPallet::max_redeem(underlying_asset_id, &ALICE), Ok(0));
		assert_eq!(
			VaultPallet::preview_redeem(underlying_asset_id, 1e18 as Balance),
			Ok(1e18 as Balance)
		);

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_ok!(VaultPallet::redeem(Origin::signed(ALICE), ASSET1, 0, ALICE,));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);

		assert_eq!(underlying_balance_before, underlying_balance_after);
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 0);
	})
}

fn prepare_vault_with_shares_but_no_assets(
	underlying_asset_id: CurrencyId,
	share_amounts: Balance,
) -> CurrencyId {
	assert_ok!(VaultPallet::create_vault_asset(
		Origin::root(),
		underlying_asset_id,
		TOKEN1_DECIMAL,
		TOKEN1_DECIMAL,
		5e17 as Balance,
		0,
	));

	let vault_asset_id = VaultPallet::vault_asset(underlying_asset_id).unwrap();

	set_balance(vault_asset_id, share_amounts, &ALICE);

	vault_asset_id
}

#[test]
fn vault_with_shares_but_no_assets() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		prepare_vault_with_shares_but_no_assets(underlying_asset_id, 1e18 as Balance);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(0));
	})
}

#[test]
fn deposit_vault_with_shares_but_no_assets() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		let mint_shares = 1e18 as Balance;
		let vault_asset_id =
			prepare_vault_with_shares_but_no_assets(underlying_asset_id, mint_shares);
		assert_eq!(VaultPallet::max_deposit(underlying_asset_id, &ALICE), Ok(0));
		assert_noop!(
			VaultPallet::preview_deposit(underlying_asset_id, 1e18 as Balance),
			Error::<Test>::Math
		);

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_noop!(
			VaultPallet::deposit(Origin::signed(ALICE), ASSET1, 1e18 as Balance, ALICE,),
			Error::<Test>::ExceedMaxDeposit
		);

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);

		assert_eq!(underlying_balance_before, underlying_balance_after);
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), mint_shares);
	})
}

#[test]
fn mint_vault_with_shares_but_no_assets() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		let mint_shares = 1e18 as Balance;
		let vault_asset_id =
			prepare_vault_with_shares_but_no_assets(underlying_asset_id, mint_shares);
		assert_eq!(VaultPallet::max_mint(underlying_asset_id, &ALICE), Ok(Balance::MAX));
		assert_eq!(VaultPallet::preview_mint(underlying_asset_id, 1e18 as Balance), Ok(0));

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_ok!(VaultPallet::mint(Origin::signed(ALICE), ASSET1, 1e18 as Balance, BOB,));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);

		assert_eq!(underlying_balance_before, underlying_balance_after);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), mint_shares);
	})
}

#[test]
fn withdraw_vault_with_shares_but_no_assets() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		let mint_shares = 1e18 as Balance;
		let vault_asset_id =
			prepare_vault_with_shares_but_no_assets(underlying_asset_id, mint_shares);
		assert_eq!(VaultPallet::max_withdraw(underlying_asset_id, &ALICE), Ok(0));
		assert_noop!(
			VaultPallet::preview_withdraw(underlying_asset_id, 1e18 as Balance),
			Error::<Test>::Math
		);

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_ok!(VaultPallet::withdraw(Origin::signed(ALICE), ASSET1, 0, BOB,));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);

		assert_eq!(underlying_balance_before, underlying_balance_after);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 0);
	})
}

#[test]
fn redeem_vault_with_shares_but_no_assets() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		let mint_shares = 1e18 as Balance;
		let vault_asset_id =
			prepare_vault_with_shares_but_no_assets(underlying_asset_id, mint_shares);

		assert_eq!(VaultPallet::max_redeem(underlying_asset_id, &ALICE), Ok(1e18 as Balance));
		assert_eq!(VaultPallet::preview_redeem(underlying_asset_id, 0), Ok(0));

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_ok!(VaultPallet::redeem(
			Origin::signed(ALICE),
			underlying_asset_id,
			1e18 as Balance,
			BOB,
		));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);

		assert_eq!(underlying_balance_before, underlying_balance_after);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 0);
	})
}

fn prepare_ongoing_vault(
	underlying_asset_id: CurrencyId,
	amounts: Balance,
	shares: Balance,
) -> CurrencyId {
	assert_ok!(VaultPallet::create_vault_asset(
		Origin::root(),
		underlying_asset_id,
		TOKEN1_DECIMAL,
		TOKEN1_DECIMAL,
		5e17 as Balance,
		0,
	));

	let vault_asset_id = VaultPallet::vault_asset(underlying_asset_id).unwrap();

	transfer_from(underlying_asset_id, &ALICE, amounts, &VAULT_PALLET_ACCOUNT);
	set_balance(vault_asset_id, shares, &ALICE);

	vault_asset_id
}

#[test]
fn create_ongoing_vault_should_work() {
	new_test_ext().execute_with(|| {
		let amounts = 1e18 as Balance;
		let shares = 1e20 as Balance;
		prepare_ongoing_vault(ASSET1, amounts, shares);
		assert_eq!(VaultPallet::total_assets(ASSET1), Ok(amounts));
	});
}

#[test]
fn deposit_with_ongoing_vault_should_work() {
	new_test_ext().execute_with(|| {
		let amounts = 1e18 as Balance;
		let shares = 1e20 as Balance;
		let underlying_asset_id = ASSET1;
		let vault_asset = prepare_ongoing_vault(ASSET1, amounts, shares);

		assert_eq!(VaultPallet::max_deposit(underlying_asset_id, &ALICE), Ok(Balance::MAX));
		assert_eq!(VaultPallet::preview_deposit(underlying_asset_id, amounts), Ok(1e20 as Balance));

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_ok!(VaultPallet::deposit(Origin::signed(ALICE), underlying_asset_id, amounts, BOB));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);
		assert_eq!(underlying_balance_before - underlying_balance_after, amounts);
		assert_eq!(get_user_balance(vault_asset, &BOB), shares);
	});
}

#[test]
fn mint_with_ongoing_vault_should_work() {
	new_test_ext().execute_with(|| {
		let amounts = 1e18 as Balance;
		let shares = 1e20 as Balance;
		let underlying_asset_id = ASSET1;
		let vault_asset = prepare_ongoing_vault(ASSET1, amounts, shares);

		assert_eq!(VaultPallet::max_mint(underlying_asset_id, &ALICE), Ok(Balance::MAX));
		assert_eq!(VaultPallet::preview_mint(underlying_asset_id, shares), Ok(1e18 as Balance));

		let underlying_balance_before = get_user_balance(underlying_asset_id, &ALICE);

		assert_ok!(VaultPallet::mint(Origin::signed(ALICE), underlying_asset_id, shares, BOB));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &ALICE);
		assert_eq!(underlying_balance_before - underlying_balance_after, amounts);
		assert_eq!(get_user_balance(vault_asset, &BOB), shares);
	});
}

#[test]
fn withdraw_with_ongoing_vault_should_work() {
	new_test_ext().execute_with(|| {
		let amounts = 1e18 as Balance;
		let shares = 1e20 as Balance;
		let underlying_asset_id = ASSET1;
		let vault_asset = prepare_ongoing_vault(ASSET1, amounts, shares);

		let ratio = VaultPallet::withdraw_fee_ratio(underlying_asset_id).unwrap();
		let expected_underlying_asset_received = amounts - amounts * ratio / amounts;

		assert_eq!(VaultPallet::max_withdraw(underlying_asset_id, &ALICE), Ok(amounts));
		assert_eq!(VaultPallet::preview_withdraw(underlying_asset_id, amounts), Ok(shares));

		let underlying_balance_before = get_user_balance(underlying_asset_id, &BOB);

		assert_ok!(VaultPallet::withdraw(Origin::signed(ALICE), underlying_asset_id, amounts, BOB));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &BOB);
		assert_eq!(
			underlying_balance_after - underlying_balance_before,
			expected_underlying_asset_received
		);
		assert_eq!(get_user_balance(vault_asset, &BOB), 0);
	});
}

#[test]
fn redeem_with_ongoing_vault_should_work() {
	new_test_ext().execute_with(|| {
		let amounts = 1e18 as Balance;
		let shares = 1e20 as Balance;
		let underlying_asset_id = ASSET1;
		let vault_asset = prepare_ongoing_vault(ASSET1, amounts, shares);

		let ratio = VaultPallet::withdraw_fee_ratio(underlying_asset_id).unwrap();
		let expected_underlying_asset_received = amounts - amounts * ratio / amounts;

		assert_eq!(VaultPallet::max_redeem(underlying_asset_id, &ALICE), Ok(shares));
		assert_eq!(VaultPallet::preview_redeem(underlying_asset_id, shares), Ok(amounts));

		let underlying_balance_before = get_user_balance(underlying_asset_id, &BOB);

		assert_ok!(VaultPallet::redeem(Origin::signed(ALICE), underlying_asset_id, shares, BOB));

		let underlying_balance_after = get_user_balance(underlying_asset_id, &BOB);
		assert_eq!(
			underlying_balance_after - underlying_balance_before,
			expected_underlying_asset_received
		);
		assert_eq!(get_user_balance(vault_asset, &BOB), 0);
	});
}

#[test]
fn mixed_transactions_should_work() {
	new_test_ext().execute_with(|| {
		let underlying_asset_id = ASSET1;
		assert_ok!(VaultPallet::create_vault_asset(
			Origin::root(),
			ASSET1,
			TOKEN1_DECIMAL,
			TOKEN1_DECIMAL,
			5e17 as Balance,
			0,
		));

		let vault_asset_id = VaultPallet::vault_asset(underlying_asset_id).unwrap();

		// 1. Alice mints 2000 shares (costs 2000 tokens)
		assert_ok!(VaultPallet::mint(Origin::signed(ALICE), underlying_asset_id, 2000, ALICE));
		assert_eq!(VaultPallet::preview_deposit(underlying_asset_id, 2000), Ok(2000));
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 2000);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 0);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 2000), Ok(2000));
		assert_eq!(<Test as Config>::MultiAsset::total_issuance(vault_asset_id), 2000);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(2000));

		// 2. Bob deposit 4000 tokens (mint 4000 shares)
		assert_ok!(VaultPallet::deposit(Origin::signed(BOB), underlying_asset_id, 4000, BOB));
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 4000);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 2000), Ok(2000));
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 4000), Ok(4000));
		assert_eq!(<Test as Config>::MultiAsset::total_issuance(vault_asset_id), 6000);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(6000));

		// 3. Vault mutates by +3000 tokens (simulated yield returned from strategy)
		transfer_from(underlying_asset_id, &ALICE, 3000, &VAULT_PALLET_ACCOUNT);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 2000), Ok(3000));
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 4000), Ok(6000));
		assert_eq!(<Test as Config>::MultiAsset::total_issuance(vault_asset_id), 6000);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(9000));

		// 4. Alice deposits 2000 tokens (mints 1333 shares)
		assert_ok!(VaultPallet::deposit(Origin::signed(ALICE), underlying_asset_id, 2000, ALICE));
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 3333);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 3333), Ok(4999));
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 4000), Ok(6000));
		assert_eq!(<Test as Config>::MultiAsset::total_issuance(vault_asset_id), 7333);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(11000));

		// 5. Bob mints 2000 shares (costs 3001 assets)
		assert_ok!(VaultPallet::mint(Origin::signed(BOB), underlying_asset_id, 2000, BOB));
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 3333);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 6000);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 3333), Ok(5000));
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 6000), Ok(9000));
		assert_eq!(<Test as Config>::MultiAsset::total_issuance(vault_asset_id), 9333);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(14001));

		// 6. Vault mutates by +3000 tokens
		transfer_from(underlying_asset_id, &ALICE, 3000, &VAULT_PALLET_ACCOUNT);
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 3333);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 6000);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 3333), Ok(6071));
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 6000), Ok(10929));
		assert_eq!(<Test as Config>::MultiAsset::total_issuance(vault_asset_id), 9333);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(17001));

		// 7. Alice redeem 1333 shares (2428 assets)
		assert_ok!(VaultPallet::redeem(Origin::signed(ALICE), underlying_asset_id, 1333, ALICE));
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 2000);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 6000);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 2000), Ok(3946));
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 6000), Ok(11840));
		assert_eq!(<Test as Config>::MultiAsset::total_issuance(vault_asset_id), 8000);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(15787));

		// 8. Bob withdraws 2929 assets (1485 shares)
		assert_eq!(VaultPallet::preview_withdraw(underlying_asset_id, 2929), Ok(1485));
		assert_ok!(VaultPallet::withdraw(Origin::signed(BOB), underlying_asset_id, 2929, BOB));
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 2000);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 4515);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 2000), Ok(4396));
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 4515), Ok(9925));
		assert_eq!(<Test as Config>::MultiAsset::total_issuance(vault_asset_id), 6515);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(14322));

		// 9. Alice withdraws 4396 assets (2000 shares)
		assert_eq!(VaultPallet::preview_withdraw(underlying_asset_id, 4396), Ok(2000));
		assert_ok!(VaultPallet::withdraw(Origin::signed(ALICE), underlying_asset_id, 4396, ALICE));
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 0);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 4515);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 4515), Ok(12124));

		// 10. Bob redeem 4515 shares (12124 tokens)
		assert_eq!(VaultPallet::preview_redeem(underlying_asset_id, 4515), Ok(12124));
		assert_ok!(VaultPallet::redeem(Origin::signed(BOB), underlying_asset_id, 4515, BOB));
		assert_eq!(get_user_balance(vault_asset_id, &ALICE), 0);
		assert_eq!(get_user_balance(vault_asset_id, &BOB), 0);
		assert_eq!(VaultPallet::convert_to_assets(underlying_asset_id, 0), Ok(0));
		assert_eq!(<Test as Config>::MultiAsset::total_issuance(vault_asset_id), 0);
		assert_eq!(VaultPallet::total_assets(underlying_asset_id), Ok(6062));
	})
}

#[test]
pub fn overflow_checked() {
	let a = u128::MAX;
	let b = 100;
	let c = 200;

	assert_eq!(balance_mul_div(a, b, c, sp_arithmetic::Rounding::Down), Some(u128::MAX / 2));
	assert_eq!(balance_mul_div(a, b, c, sp_arithmetic::Rounding::Up), Some(u128::MAX / 2 + 1));
}
