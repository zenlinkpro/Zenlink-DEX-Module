use frame_support::{assert_noop, assert_ok};

use super::{AssetId, Error, mock::*};

const DOT: AssetId = AssetId::ParaCurrency(0);
const BTC: AssetId = AssetId::ParaCurrency(1);
const DEV: AssetId = AssetId::NativeCurrency;


const PAIR_ACCOUNT_0: u128 = 15310315390164549602772283245;

const ALICE: u128 = 1;
const BOB: u128 = 2;

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

        let total_supply_dot = u128::MAX / 2;
        let total_supply_btc = u128::MAX / 4;

        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_btc,
			total_supply_dot,
			0,
			0
		));

        let total_supply_dot = 1_00u128;
        let total_supply_btc = 1_00u128 / 2;

        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE,
			&BTC,
			&DOT,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));

        let balance_dot = DexModule::balance_of(DOT, &PAIR_ACCOUNT_0);
        let balance_btc = DexModule::balance_of(BTC, &PAIR_ACCOUNT_0);
        println!("balance_DOT {}, balance_BTC {}", balance_dot, balance_btc);
    });
}

#[test]
fn inner_get_in_price_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_mint(DOT, &ALICE, 2000_000_000_00u128));
        assert_ok!(DexModule::inner_mint(BTC, &ALICE, 1000_000_000_00u128));
        assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
        assert_ok!(DexModule::inner_create_pair(&BTC, &DEV));

        let total_supply_dot = 2000_000_0000u128;
        let total_supply_btc = 1000_000_0000u128;

        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_btc,
			total_supply_dot,
			0,
			0
		));

        let balance_dot = DexModule::balance_of(DOT, &PAIR_ACCOUNT_0);
        let balance_btc = DexModule::balance_of(BTC, &PAIR_ACCOUNT_0);
        println!("balance_dot:{} balance_btc{}", balance_dot, balance_btc);
        let path = vec![DOT.clone(), BTC.clone()];
        let amount_in_unit = 1_000_000_00u128;

        let _amounts = DexModule::get_amount_out_by_path(amount_in_unit, &path);
    });
}

#[test]
fn inner_add_liquidity_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_mint(DOT, &ALICE, 100000));
        assert_ok!(DexModule::inner_mint(BTC, &ALICE, 100000));

        assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));

        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE, &DOT, &BTC, 1000u128, 2000u128, 1500u128, 1000u128
		));

        assert_noop!(
			DexModule::inner_add_liquidity_local(&ALICE, &DOT, &BTC, 2000u128, 2000u128, 2000u128, 2000u128),
            Error::<Test>::IncorrectAssetAmountRange
		);
    })
}

#[test]
fn remove_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_mint(DOT, &ALICE, 100000u128));
        assert_ok!(DexModule::inner_mint(BTC, &ALICE, 100000u128));

        assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));

        let total_supply_dot = 4000u128;
        let total_supply_btc = 2000u128;
        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));

        assert_ok!(DexModule::inner_remove_liquidity(
			&ALICE, &BTC, &DOT, 10u128, 0u128, 0u128, &BOB
		));

        let balance_dot = DexModule::balance_of(DOT, &BOB);
        let balance_btc = DexModule::balance_of(BTC, &BOB);

        println!("balance_dot {}, balance_btc {}", balance_dot, balance_btc);
    })
}

#[test]
fn inner_swap_exact_tokens_for_tokens_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_mint(DOT, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_mint(BTC, &ALICE, u128::MAX));

        assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));

        let total_supply_dot = u128::MAX / 2;
        let total_supply_btc = u128::MAX / 4;

        assert_ok!(DexModule::inner_add_liquidity_local(
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
        let amount_in = 1000000000u128;
        let amount_out_min = 1u128;
        assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens_local(
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
        let amount_in = 1000000000u128;
        let amount_out_min = 1u128;
        assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens_local(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));
        let dot_balance = DexModule::balance_of(DOT, &BOB);
        println!("dot_balance {}", dot_balance);
        assert!(dot_balance > amount_out_min);
    })
}

#[test]
fn inner_swap_exact_tokens_for_tokens_in_pairs_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_mint(DOT, &ALICE, u128::MAX));
        assert_ok!(DexModule::inner_mint(BTC, &ALICE, u128::MAX));

        assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
        assert_ok!(DexModule::inner_create_pair(&BTC, &DEV));

        let total_supply_dot = 100000000000000000000;
        let total_supply_btc = total_supply_dot / 2;
        let total_supply_dev = total_supply_btc / 2;

        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE,
			&BTC,
			&DEV,
			total_supply_btc,
			total_supply_dev,
			0,
			0
		));

        let path = vec![DOT.clone(), BTC.clone(), DEV.clone()];
        let amount_in = 600u128;
        let amount_out_min = 1u128;
        let dot_balance = DexModule::asset_balance_of(&DEV, &BOB);
        println!("dot_balance {}", dot_balance);

        assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens_local(
			&ALICE,
			amount_in,
			amount_out_min,
			&path,
			&BOB,
		));
        let dot_balance = DexModule::asset_balance_of(&DEV, &BOB);

        println!("dot_balance {}", dot_balance);

        let path = vec![DEV.clone(), BTC.clone(), DOT.clone()];
        let amount_in = 600u128;
        let amount_out_min = 1u128;
        let dot_balance = DexModule::asset_balance_of(&DEV, &BOB);
        println!("dot_balance {}", dot_balance);

        assert_ok!(DexModule::inner_swap_exact_tokens_for_tokens_local(
            &ALICE,
            amount_in,
            amount_out_min,
            &path,
            &BOB,
        ));
        let dot_balance = DexModule::asset_balance_of(&DOT, &BOB);

        println!("dot_balance {}", dot_balance);

        assert!(dot_balance > amount_out_min);
    })
}

#[test]
fn inner_swap_tokens_for_exact_tokens_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_mint(DOT, &ALICE,  u128::MAX));
        assert_ok!(DexModule::inner_mint(BTC, &ALICE,  u128::MAX));

        assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));

        let total_supply_dot = 10000000000000000;
        let total_supply_btc = total_supply_dot / 2;

        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
        let path = vec![DOT.clone(), BTC.clone()];
        let amount_out = 1000000000u128;
        let amount_in_max = 3000000000u128;
        assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens_local(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
        let btc_balance = DexModule::balance_of(BTC, &BOB);
        println!("dot_balance {}", btc_balance);
        assert_eq!(btc_balance, amount_out);

        let path = vec![BTC.clone(), DOT.clone()];
        let amount_out = 1000000000u128;
        let amount_in_max = 3000000000u128;
        assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens_local(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
        let dot_balance = DexModule::asset_balance_of(&DOT, &BOB);
        println!("dot_balance {}", dot_balance);
        assert_eq!(dot_balance, amount_out);
    })
}

#[test]
fn inner_swap_tokens_for_exact_tokens_in_pairs_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(DexModule::inner_mint(DOT, &ALICE, u128::MAX /2 ));
        assert_ok!(DexModule::inner_mint(BTC, &ALICE, u128::MAX /2 ));
        //assert_ok!(DexModule::inner_mint(DEV, &ALICE, 100000u128));
        let dev_balance_ori = DexModule::asset_balance_of(&DEV, &BOB);
        assert_ok!(DexModule::inner_create_pair(&DOT, &BTC));
        assert_ok!(DexModule::inner_create_pair(&BTC, &DEV));

        let total_supply_dot = 100000000000000;
        let total_supply_btc = total_supply_dot / 2;
        let total_supply_dev = total_supply_btc / 2;

        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE,
			&DOT,
			&BTC,
			total_supply_dot,
			total_supply_btc,
			0,
			0
		));
        assert_ok!(DexModule::inner_add_liquidity_local(
			&ALICE,
			&BTC,
			&DEV,
			total_supply_btc,
			total_supply_dev,
			0,
			0
		));

        let path = vec![DOT.clone(), BTC.clone(), DEV.clone()];
        let amount_out = 10000u128;
        let amount_in_max = 45000u128;
        assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens_local(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
        let dev_balance = DexModule::asset_balance_of(&DEV, &BOB);

        println!("dev_balance {}", dev_balance);
        assert_eq!(dev_balance - dev_balance_ori, amount_out);

        let path = vec![DEV.clone(), BTC.clone(), DOT.clone()];
        let amount_out = 10000u128;
        let amount_in_max = 90000u128;
        assert_ok!(DexModule::inner_swap_tokens_for_exact_tokens_local(
			&ALICE,
			amount_out,
			amount_in_max,
			&path,
			&BOB
		));
        let dot_balance = DexModule::asset_balance_of(&DOT, &BOB);

        println!("dev_balance {}", dot_balance);
        assert_eq!(dot_balance, amount_out);
    })
}