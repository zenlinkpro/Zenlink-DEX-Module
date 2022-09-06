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

