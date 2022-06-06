use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError::BadOrigin;

use super::{
	mock::{CurrencyId::*, PoolToken::Token as pool_token, PoolType::*, *},
	*,
};

const POOL0ACCOUNTED: AccountId = 33865947477506447919519395693;

type MockPool = Pool<CurrencyId, AccountId>;

const INITIAL_A_VALUE: Balance = 50;
const SWAP_FEE: Balance = 1e7 as Balance;
const ADMIN_FEE: Balance = 0;

const BASIC_4_POOL_CURRENCY_ID: CurrencyId = StableLP(P4(
	pool_token(TOKEN1_SYMBOL),
	pool_token(TOKEN2_SYMBOL),
	pool_token(TOKEN3_SYMBOL),
	pool_token(TOKEN4_SYMBOL),
));

const BASIC_3_POOL_CURRENCY_ID: CurrencyId = StableLP(P3(
	pool_token(TOKEN1_SYMBOL),
	pool_token(TOKEN2_SYMBOL),
	pool_token(TOKEN3_SYMBOL),
));

const BASIC_2_POOL_CURRENCY_ID: CurrencyId = StableLP(P2(pool_token(TOKEN1_SYMBOL), pool_token(TOKEN2_SYMBOL)));

#[test]
fn create_pool_with_incorrect_parameter_should_not_work() {
	new_test_ext().execute_with(|| {
		// only root can create pool
		assert_noop!(
			StableAmm::create_pool(
				Origin::signed(ALICE),
				vec![
					Token(TOKEN1_SYMBOL),
					Token(TOKEN2_SYMBOL),
					Token(TOKEN3_SYMBOL),
					Token(TOKEN4_SYMBOL)
				],
				vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL, TOKEN4_DECIMAL],
				BASIC_4_POOL_CURRENCY_ID,
				0,
				0,
				0
			),
			BadOrigin
		);
		assert_eq!(StableAmm::next_pool_id(), 0);
		assert_eq!(StableAmm::pools(0), None);

		//create mismatch parameter should not work
		assert_noop!(
			StableAmm::create_pool(
				Origin::root(),
				vec![Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL), Token(TOKEN3_SYMBOL),],
				vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL, TOKEN4_DECIMAL],
				BASIC_3_POOL_CURRENCY_ID,
				0,
				0,
				0
			),
			Error::<Test>::MismatchParameter
		);
		assert_eq!(StableAmm::next_pool_id(), 0);
		assert_eq!(StableAmm::pools(0), None);

		// create with forbidden token should not work
		assert_noop!(
			StableAmm::create_pool(
				Origin::root(),
				vec![
					Forbidden(TOKEN1_SYMBOL),
					Token(TOKEN2_SYMBOL),
					Token(TOKEN3_SYMBOL),
					Token(TOKEN4_SYMBOL)
				],
				vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL, TOKEN4_DECIMAL],
				BASIC_3_POOL_CURRENCY_ID,
				0,
				0,
				0
			),
			Error::<Test>::InvalidPooledCurrency
		);
		assert_eq!(StableAmm::next_pool_id(), 0);
		assert_eq!(StableAmm::pools(0), None);

		// Create with incorrect lp token should not work
		assert_noop!(
			StableAmm::create_pool(
				Origin::root(),
				vec![Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL), Token(TOKEN3_SYMBOL),],
				vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL],
				Token(TOKEN4_SYMBOL),
				0,
				0,
				0
			),
			Error::<Test>::InvalidLpCurrency
		);
		assert_eq!(StableAmm::next_pool_id(), 0);
		assert_eq!(StableAmm::pools(0), None);

		// Create with invalid decimal should not work
		assert_noop!(
			StableAmm::create_pool(
				Origin::root(),
				vec![Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL), Token(TOKEN3_SYMBOL),],
				vec![TOKEN1_DECIMAL, 20, TOKEN3_DECIMAL],
				BASIC_3_POOL_CURRENCY_ID,
				0,
				0,
				0
			),
			Error::<Test>::InvalidCurrencyDecimal
		);
		assert_eq!(StableAmm::next_pool_id(), 0);
		assert_eq!(StableAmm::pools(0), None);
	});
}

#[test]
fn create_pool_with_parameters_exceed_threshold_should_not_work() {
	new_test_ext().execute_with(|| {
		let lp_currency_id = BASIC_4_POOL_CURRENCY_ID;
		// exceed max swap fee
		assert_noop!(
			StableAmm::create_pool(
				Origin::root(),
				vec![
					Token(TOKEN1_SYMBOL),
					Token(TOKEN2_SYMBOL),
					Token(TOKEN3_SYMBOL),
					Token(TOKEN4_SYMBOL)
				],
				vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL, TOKEN4_DECIMAL],
				lp_currency_id,
				0,
				MAX_SWAP_FEE.into(),
				0,
			),
			Error::<Test>::ExceedMaxFee
		);
		assert_eq!(StableAmm::next_pool_id(), 0);
		assert_eq!(StableAmm::pools(0), None);

		// exceed max fee
		assert_noop!(
			StableAmm::create_pool(
				Origin::root(),
				vec![
					Token(TOKEN1_SYMBOL),
					Token(TOKEN2_SYMBOL),
					Token(TOKEN3_SYMBOL),
					Token(TOKEN4_SYMBOL)
				],
				vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL, TOKEN4_DECIMAL],
				lp_currency_id,
				0,
				(MAX_SWAP_FEE - 1).into(),
				MAX_ADMIN_FEE.into(),
			),
			Error::<Test>::ExceedMaxFee
		);
		assert_eq!(StableAmm::next_pool_id(), 0);
		assert_eq!(StableAmm::pools(0), None);

		// exceed max a
		assert_noop!(
			StableAmm::create_pool(
				Origin::root(),
				vec![
					Token(TOKEN1_SYMBOL),
					Token(TOKEN2_SYMBOL),
					Token(TOKEN3_SYMBOL),
					Token(TOKEN4_SYMBOL)
				],
				vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL, TOKEN4_DECIMAL],
				lp_currency_id,
				MAX_A.into(),
				(MAX_SWAP_FEE - 1).into(),
				(MAX_ADMIN_FEE - 1).into(),
			),
			Error::<Test>::ExceedMaxA
		);
		assert_eq!(StableAmm::next_pool_id(), 0);
		assert_eq!(StableAmm::pools(0), None);
	});
}

#[test]
fn create_pool_with_already_used_lp_currency_should_not_work() {
	new_test_ext().execute_with(|| {
		let lp_currency_id = BASIC_4_POOL_CURRENCY_ID;
		assert_eq!(StableAmm::lp_currencies(lp_currency_id), None);

		assert_ok!(StableAmm::create_pool(
			Origin::root(),
			vec![Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL), Token(TOKEN3_SYMBOL)],
			vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL],
			lp_currency_id,
			(MAX_A - 1).into(),
			(MAX_SWAP_FEE - 1).into(),
			(MAX_ADMIN_FEE - 1).into()
		));

		assert_eq!(StableAmm::next_pool_id(), 1);
		assert_eq!(StableAmm::lp_currencies(lp_currency_id), Some(0));

		assert_noop!(
			StableAmm::create_pool(
				Origin::root(),
				vec![Token(TOKEN2_SYMBOL), Token(TOKEN3_SYMBOL), Token(TOKEN4_SYMBOL)],
				vec![TOKEN2_DECIMAL, TOKEN3_DECIMAL, TOKEN4_DECIMAL],
				lp_currency_id,
				(MAX_A - 1).into(),
				(MAX_SWAP_FEE - 1).into(),
				(MAX_ADMIN_FEE - 1).into()
			),
			Error::<Test>::LpCurrencyAlreadyUsed
		);

		assert_eq!(StableAmm::pools(1), None);
		assert_eq!(StableAmm::next_pool_id(), 1);
		assert_eq!(StableAmm::lp_currencies(lp_currency_id), Some(0))
	})
}

#[test]
fn create_pool_should_work() {
	new_test_ext().execute_with(|| {
		let lp_currency_id = BASIC_4_POOL_CURRENCY_ID;
		assert_eq!(StableAmm::lp_currencies(lp_currency_id), None);

		assert_ok!(StableAmm::create_pool(
			Origin::root(),
			vec![
				Token(TOKEN1_SYMBOL),
				Token(TOKEN2_SYMBOL),
				Token(TOKEN3_SYMBOL),
				Token(TOKEN4_SYMBOL)
			],
			vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL, TOKEN4_DECIMAL],
			lp_currency_id,
			INITIAL_A_VALUE,
			SWAP_FEE,
			ADMIN_FEE
		));

		assert_eq!(StableAmm::next_pool_id(), 1);

		assert_eq!(
			StableAmm::pools(0),
			Some(MockPool {
				pooled_currency_ids: vec![
					Token(TOKEN1_SYMBOL),
					Token(TOKEN2_SYMBOL),
					Token(TOKEN3_SYMBOL),
					Token(TOKEN4_SYMBOL),
				],
				lp_currency_id,
				token_multipliers: vec![
					checked_pow(10, (POOL_TOKEN_COMMON_DECIMALS - TOKEN1_DECIMAL) as usize).unwrap(),
					checked_pow(10, (POOL_TOKEN_COMMON_DECIMALS - TOKEN2_DECIMAL) as usize).unwrap(),
					checked_pow(10, (POOL_TOKEN_COMMON_DECIMALS - TOKEN3_DECIMAL) as usize).unwrap(),
					checked_pow(10, (POOL_TOKEN_COMMON_DECIMALS - TOKEN4_DECIMAL) as usize).unwrap(),
				],
				balances: vec![Zero::zero(); 4],
				fee: SWAP_FEE,
				admin_fee: ADMIN_FEE,
				initial_a: INITIAL_A_VALUE * (A_PRECISION as Balance),
				future_a: INITIAL_A_VALUE * (A_PRECISION as Balance),
				initial_a_time: 0,
				future_a_time: 0,
				pool_account: POOL0ACCOUNTED,
			})
		);

		assert_eq!(StableAmm::lp_currencies(lp_currency_id), Some(0))
	});
}

#[test]
fn add_liquidity_with_incorrect_should_not_work() {
	new_test_ext().execute_with(|| {
		let lp_currency_id = BASIC_2_POOL_CURRENCY_ID;
		assert_ok!(StableAmm::create_pool(
			Origin::root(),
			vec![Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL),],
			vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL],
			lp_currency_id,
			INITIAL_A_VALUE,
			SWAP_FEE,
			ADMIN_FEE
		));

		// case0: add_liquidity with incorrect pool id
		assert_noop!(
			StableAmm::add_liquidity(
				Origin::signed(BOB),
				1,
				vec![1e16 as Balance, 2e18 as Balance],
				0,
				u64::MAX,
			),
			Error::<Test>::InvalidPoolId
		);

		// case1: add_liquidity with invalid amounts length
		assert_noop!(
			StableAmm::add_liquidity(Origin::signed(BOB), 0, vec![1e16 as Balance], 0, u64::MAX,),
			Error::<Test>::InvalidParameter
		);

		// case2: initial add liquidity require all currencies
		assert_noop!(
			StableAmm::add_liquidity(Origin::signed(BOB), 0, vec![1e16 as Balance, 0 as Balance], 0, u64::MAX,),
			Error::<Test>::RequireAllCurrencies
		);
	});
}

#[test]
fn add_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		let lp_currency_id = BASIC_2_POOL_CURRENCY_ID;
		assert_ok!(StableAmm::create_pool(
			Origin::root(),
			vec![Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL),],
			vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL],
			lp_currency_id,
			INITIAL_A_VALUE,
			SWAP_FEE,
			ADMIN_FEE
		));

		// case0: add liquidity should work.
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(ALICE),
			0,
			vec![1e18 as Balance, 1e18 as Balance],
			0,
			u64::MAX,
		));
		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(lp_currency_id, &ALICE),
			2e18 as Balance
		);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![1e18 as Balance, 3e18 as Balance],
			0,
			u64::MAX,
		));
		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB),
			3991672211258372957
		);
	});
}

#[test]
fn add_liquidity_with_expected_amount_lp_token_should_work() {
	new_test_ext().execute_with(|| {
		let lp_currency_id = BASIC_2_POOL_CURRENCY_ID;
		assert_ok!(StableAmm::create_pool(
			Origin::root(),
			vec![Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL),],
			vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL],
			lp_currency_id,
			INITIAL_A_VALUE,
			SWAP_FEE,
			ADMIN_FEE
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(ALICE),
			0,
			vec![1e18 as Balance, 1e18 as Balance],
			0,
			u64::MAX,
		));

		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(lp_currency_id, &ALICE),
			2e18 as Balance
		);
		let calculated_lp_token_amount =
			StableAmm::calculate_token_amount(0, vec![1e18 as Balance, 3e18 as Balance], true).unwrap_or_default();
		assert_eq!(calculated_lp_token_amount, 3992673697878079065);

		let calculated_lp_token_amount_with_slippage = calculated_lp_token_amount*999/1000;

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![1e18 as Balance, 3e18 as Balance],
			calculated_lp_token_amount_with_slippage,
			u64::MAX,
		));
		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB),
			3991672211258372957
		);
	});
}
