use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError::BadOrigin;

use super::{
	mock::{CurrencyId::*, PoolToken::Token as pool_token, PoolType::*, *},
	*,
};

const POOL0ACCOUNTID: AccountId = 33865947477506447919519395693;

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

fn setup_test_pool() -> (PoolId, CurrencyId) {
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
	(0, lp_currency_id)
}

fn get_user_token_balances(currencies: &[CurrencyId], user: &AccountId) -> Vec<Balance> {
	let mut res = Vec::new();
	for currency_id in currencies.iter() {
		res.push(<Test as Config>::MultiCurrency::free_balance(*currency_id, user));
	}
	res
}

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
				pool_account: POOL0ACCOUNTID,
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
		let (pool_id, lp_currency_id) = setup_test_pool();

		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(lp_currency_id, &ALICE),
			2e18 as Balance
		);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
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
		let (pool_id, lp_currency_id) = setup_test_pool();

		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(lp_currency_id, &ALICE),
			2e18 as Balance
		);
		let calculated_lp_token_amount =
			StableAmm::calculate_token_amount(pool_id, vec![1e18 as Balance, 3e18 as Balance], true)
				.unwrap_or_default();
		assert_eq!(calculated_lp_token_amount, 3992673697878079065);

		let calculated_lp_token_amount_with_slippage = calculated_lp_token_amount * 999 / 1000;

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
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

#[test]
fn add_liquidity_lp_token_amount_has_small_slippage_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, lp_currency_id) = setup_test_pool();

		let calculated_lp_token_amount =
			StableAmm::calculate_token_amount(pool_id, vec![1e18 as Balance, 3e18 as Balance], true)
				.unwrap_or_default();

		let calculated_lp_token_amount_with_negative_slippage = calculated_lp_token_amount * 999 / 1000;
		let calculated_lp_token_amount_with_positive_slippage = calculated_lp_token_amount * 1001 / 1000;
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 3e18 as Balance],
			calculated_lp_token_amount_with_negative_slippage,
			u64::MAX,
		));

		let lp_token_balance = <Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB);
		assert!(lp_token_balance > calculated_lp_token_amount_with_negative_slippage);
		assert!(lp_token_balance < calculated_lp_token_amount_with_positive_slippage);
	})
}

#[test]
fn add_liquidity_update_pool_balance_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 3e18 as Balance],
			0,
			u64::MAX,
		));

		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &POOL0ACCOUNTID),
			2e18 as Balance
		);

		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &POOL0ACCOUNTID),
			4e18 as Balance
		);
	})
}

#[test]
fn add_liquidity_when_mint_amount_not_reach_due_to_front_running_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		let calculated_lp_token_amount =
			StableAmm::calculate_token_amount(pool_id, vec![1e18 as Balance, 3e18 as Balance], true)
				.unwrap_or_default();
		let calculated_lp_token_amount_with_slippage = calculated_lp_token_amount * 999 / 1000;
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(ALICE),
			pool_id,
			vec![1e18 as Balance, 3e18 as Balance],
			0,
			u64::MAX,
		));

		assert_noop!(
			StableAmm::add_liquidity(
				Origin::signed(BOB),
				pool_id,
				vec![1e18 as Balance, 3e18 as Balance],
				calculated_lp_token_amount_with_slippage,
				u64::MAX,
			),
			Error::<Test>::AmountSlippage
		);
	})
}

#[test]
fn add_liquidity_with_expired_deadline_should_not_work() {
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

		System::set_block_number(100);

		assert_noop!(
			StableAmm::add_liquidity(Origin::signed(ALICE), 0, vec![1e18 as Balance, 1e18 as Balance], 0, 99,),
			Error::<Test>::Deadline
		);
	})
}

#[test]
fn remove_liquidity_exceed_total_supply_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		assert!(StableAmm::calculate_removed_liquidity(&pool, u128::MAX) == None)
	})
}

#[test]
fn remove_liquidity_with_incorrect_min_amounts_length_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_noop!(
			StableAmm::remove_liquidity(Origin::signed(ALICE), pool_id, 2e18 as Balance, vec![0], u64::MAX,),
			Error::<Test>::InvalidParameter
		);
	})
}

#[test]
fn remove_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, lp_currency_id) = setup_test_pool();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e16 as Balance],
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::remove_liquidity(
			Origin::signed(ALICE),
			pool_id,
			2e18 as Balance,
			vec![0, 0],
			u64::MAX
		));

		let current_bob_balance = <Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB);
		assert_eq!(current_bob_balance, 1996275270169644725);

		assert_ok!(StableAmm::remove_liquidity(
			Origin::signed(BOB),
			pool_id,
			current_bob_balance,
			vec![0, 0],
			u64::MAX
		));
		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &POOL0ACCOUNTID),
			0
		);
		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &POOL0ACCOUNTID),
			0
		);
	})
}

#[test]
fn remove_liquidity_with_expected_return_amount_underlying_currency_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, lp_currency_id) = setup_test_pool();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e16 as Balance],
			0,
			u64::MAX,
		));
		let first_token_balance_before = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);
		let second_token_balance_before = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &BOB);
		let pool_token_balance_before = <Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB);

		assert_eq!(pool_token_balance_before, 1996275270169644725);
		let pool = StableAmm::pools(pool_id).unwrap();
		let expected_balances = StableAmm::calculate_removed_liquidity(&pool, pool_token_balance_before).unwrap();
		assert_eq!(expected_balances[0], 1498601924450190405);
		assert_eq!(expected_balances[1], 504529314564897436);

		assert_ok!(StableAmm::remove_liquidity(
			Origin::signed(BOB),
			pool_id,
			pool_token_balance_before,
			expected_balances.clone(),
			u64::MAX
		));

		let first_token_balance_after = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);
		let second_token_balance_after = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &BOB);

		assert_eq!(
			first_token_balance_after - first_token_balance_before,
			expected_balances[0]
		);
		assert_eq!(
			second_token_balance_after - second_token_balance_before,
			expected_balances[1]
		);
	})
}

#[test]
fn remove_liquidity_exceed_own_lp_tokens_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, lp_currency_id) = setup_test_pool();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e16 as Balance],
			0,
			u64::MAX,
		));

		let pool_token_balance = <Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB);
		assert_eq!(pool_token_balance, 1996275270169644725);
		assert_noop!(
			StableAmm::remove_liquidity(
				Origin::signed(BOB),
				pool_id,
				pool_token_balance + 1,
				vec![Balance::MAX, Balance::MAX],
				u64::MAX
			),
			Error::<Test>::AmountSlippage
		);
	})
}

#[test]
fn remove_liquidity_when_min_amounts_not_reached_due_to_front_running_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, lp_currency_id) = setup_test_pool();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e16 as Balance],
			0,
			u64::MAX,
		));

		let pool_token_balance = <Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB);
		assert_eq!(pool_token_balance, 1996275270169644725);

		let pool = StableAmm::pools(pool_id).unwrap();
		let expected_balances = StableAmm::calculate_removed_liquidity(&pool, pool_token_balance).unwrap();
		assert_eq!(expected_balances[0], 1498601924450190405);
		assert_eq!(expected_balances[1], 504529314564897436);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			pool_id,
			vec![1e16 as Balance, 2e18 as Balance],
			0,
			u64::MAX,
		));

		assert_noop!(
			StableAmm::remove_liquidity(
				Origin::signed(BOB),
				pool_id,
				pool_token_balance,
				expected_balances,
				u64::MAX
			),
			Error::<Test>::AmountSlippage
		);
	})
}

#[test]
fn remove_liquidity_with_expired_deadline_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, lp_currency_id) = setup_test_pool();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e16 as Balance],
			0,
			u64::MAX,
		));
		let pool_token_balance = <Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB);

		System::set_block_number(100);

		assert_noop!(
			StableAmm::remove_liquidity(Origin::signed(BOB), pool_id, pool_token_balance, vec![0, 0], 99),
			Error::<Test>::Deadline
		);
	})
}

#[test]
fn remove_liquidity_imbalance_with_max_burn_lp_token_amount_range_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, lp_currency_id) = setup_test_pool();
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e16 as Balance],
			0,
			u64::MAX,
		));

		// calculates amount of pool token to be burned
		let max_pool_token_amount_to_be_burned = StableAmm::calculate_token_amount(pool_id, vec![1e18 as Balance, 1e16 as Balance], false).unwrap();
		assert_eq!(1000688044155287276, max_pool_token_amount_to_be_burned);

		let max_pool_token_amount_to_be_burned_negative_slippage = max_pool_token_amount_to_be_burned *1001/1000;
		let max_pool_token_amount_to_be_burned_positive_slippage = max_pool_token_amount_to_be_burned *999/1000;
		let balance_before =
			get_user_token_balances(&[Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL), lp_currency_id], &BOB);

		assert_ok!(StableAmm::remove_liquidity_imbalance(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 1e16 as Balance],
			max_pool_token_amount_to_be_burned_negative_slippage,
			u64::MAX
		));

		let balance_after =
			get_user_token_balances(&[Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL), lp_currency_id], &BOB);

		// Check the actual returned token amounts match the requested amounts
		assert_eq!(balance_after[0] - balance_before[0], 1e18 as Balance);
		assert_eq!(balance_after[1] - balance_before[1], 1e16 as Balance);
		let actual_pool_token_burned =balance_before[2] - balance_after[2];
		assert_eq!(actual_pool_token_burned, 1000934178112841888);

		assert!(actual_pool_token_burned > max_pool_token_amount_to_be_burned_positive_slippage);
		assert!(actual_pool_token_burned < max_pool_token_amount_to_be_burned_negative_slippage);
	})
}
