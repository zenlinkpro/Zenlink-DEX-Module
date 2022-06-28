use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError::BadOrigin;
use std::time::SystemTime;
use std::u64;

use super::{
	mock::{CurrencyId::*, PoolToken::Token as pool_token, PoolType::*, *},
	*,
};

const POOL0ACCOUNTID: AccountId = 33865947477506447919519395693;

type MockPool = Pool<CurrencyId, AccountId, BoundedVec<u8, PoolCurrencySymbolLimit>>;

const INITIAL_A_VALUE: Balance = 50;
const SWAP_FEE: Balance = 1e7 as Balance;
const ADMIN_FEE: Balance = 0;
const DAYS: u64 = 86400;

fn mine_block() {
	let now = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs();

	System::set_block_number(System::block_number() + 1);
	set_block_timestamp(now);
}

fn mine_block_with_timestamp(timestamp: u64) {
	System::set_block_number(System::block_number() + 1);
	set_block_timestamp(timestamp);
}

// timestamp in second
fn set_block_timestamp(timestamp: u64) {
	Timestamp::set_timestamp(timestamp * 1000);
}

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
		ADMIN_FEE,
		ALICE,
		Vec::from("stable_pool_lp"),
		18,
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

fn setup_test_pool_and_base_pool() -> (PoolId, PoolId) {
	let first_pool_lp_currency_id = BASIC_3_POOL_CURRENCY_ID;

	assert_ok!(StableAmm::create_pool(
		Origin::root(),
		vec![Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL), Token(TOKEN3_SYMBOL)],
		vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL],
		first_pool_lp_currency_id,
		INITIAL_A_VALUE,
		SWAP_FEE,
		ADMIN_FEE,
		ALICE,
		Vec::from("basic_pool_lp"),
		18,
	));

	let second_pool_lp_currency_id = StableLP(P2(pool_token(TOKEN_LP), pool_token(TOKEN4_SYMBOL)));

	assert_ok!(StableAmm::create_pool(
		Origin::root(),
		vec![first_pool_lp_currency_id, Token(TOKEN4_SYMBOL)],
		vec![18, TOKEN4_DECIMAL],
		second_pool_lp_currency_id,
		INITIAL_A_VALUE,
		SWAP_FEE,
		ADMIN_FEE,
		ALICE,
		Vec::from("pool_lp"),
		18,
	));

	(0, 1)
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
				0,
				ALICE,
				Vec::from("stable_pool_lp"),
				18,
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
				0,
				ALICE,
				Vec::from("stable_pool_lp"),
				18,
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
				0,
				ALICE,
				Vec::from("stable_pool_lp"),
				18,
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
				0,
				ALICE,
				Vec::from("stable_pool_lp"),
				18,
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
				0,
				ALICE,
				Vec::from("stable_pool_lp"),
				18,
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
				(MAX_SWAP_FEE + 1).into(),
				0,
				ALICE,
				Vec::from("stable_pool_lp"),
				18,
			),
			Error::<Test>::ExceedMaxFee
		);
		assert_eq!(StableAmm::next_pool_id(), 0);
		assert_eq!(StableAmm::pools(0), None);

		// exceed max admin fee
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
				(MAX_SWAP_FEE).into(),
				(MAX_ADMIN_FEE + 1).into(),
				ALICE,
				Vec::from("stable_pool_lp"),
				18,
			),
			Error::<Test>::ExceedMaxAdminFee
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
				ALICE,
				Vec::from("stable_pool_lp"),
				18,
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
			(MAX_ADMIN_FEE - 1).into(),
			ALICE,
			Vec::from("stable_pool_lp"),
			18,
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
				(MAX_ADMIN_FEE - 1).into(),
				ALICE,
				Vec::from("stable_pool_lp"),
				18,
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
			ADMIN_FEE,
			ALICE,
			Vec::from("stable_pool_lp"),
			18,
		));

		assert_eq!(StableAmm::next_pool_id(), 1);

		assert_eq!(
			StableAmm::pools(0),
			Some(MockPool {
				currency_ids: vec![
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
				account: POOL0ACCOUNTID,
				admin_fee_receiver: ALICE,
				lp_currency_symbol: BoundedVec::<u8, PoolCurrencySymbolLimit>::try_from(Vec::from("stable_pool_lp"))
					.unwrap(),
				lp_currency_decimal: 18,
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
			ADMIN_FEE,
			ALICE,
			Vec::from("stable_pool_lp"),
			18,
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
			Error::<Test>::MismatchParameter
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
			StableAmm::calculate_currency_amount(pool_id, vec![1e18 as Balance, 3e18 as Balance], true)
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
			StableAmm::calculate_currency_amount(pool_id, vec![1e18 as Balance, 3e18 as Balance], true)
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
			StableAmm::calculate_currency_amount(pool_id, vec![1e18 as Balance, 3e18 as Balance], true)
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
			ADMIN_FEE,
			ALICE,
			Vec::from("stable_pool_lp"),
			18,
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
			Error::<Test>::MismatchParameter
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
fn remove_liquidity_imbalance_with_mismatch_amounts_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_noop!(
			StableAmm::remove_liquidity_imbalance(
				Origin::signed(ALICE),
				pool_id,
				vec![1e18 as Balance],
				Balance::MAX,
				u64::MAX
			),
			Error::<Test>::MismatchParameter
		);
	})
}

#[test]
fn remove_liquidity_imbalance_when_withdraw_more_than_available_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_noop!(
			StableAmm::remove_liquidity_imbalance(
				Origin::signed(ALICE),
				pool_id,
				vec![Balance::MAX, Balance::MAX],
				1,
				u64::MAX
			),
			Error::<Test>::Arithmetic
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
		let max_pool_token_amount_to_be_burned =
			StableAmm::calculate_currency_amount(pool_id, vec![1e18 as Balance, 1e16 as Balance], false).unwrap();
		assert_eq!(1000688044155287276, max_pool_token_amount_to_be_burned);

		let max_pool_token_amount_to_be_burned_negative_slippage = max_pool_token_amount_to_be_burned * 1001 / 1000;
		let max_pool_token_amount_to_be_burned_positive_slippage = max_pool_token_amount_to_be_burned * 999 / 1000;
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
		let actual_pool_token_burned = balance_before[2] - balance_after[2];
		assert_eq!(actual_pool_token_burned, 1000934178112841888);

		assert!(actual_pool_token_burned > max_pool_token_amount_to_be_burned_positive_slippage);
		assert!(actual_pool_token_burned < max_pool_token_amount_to_be_burned_negative_slippage);
	})
}

#[test]
fn remove_liquidity_imbalance_exceed_own_lp_token_amount_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, lp_currency_id) = setup_test_pool();
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e16 as Balance],
			0,
			u64::MAX,
		));

		let current_balance = <Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB);
		assert_eq!(current_balance, 1996275270169644725);

		assert_noop!(
			StableAmm::remove_liquidity_imbalance(
				Origin::signed(BOB),
				pool_id,
				vec![2e18 as Balance, 1e16 as Balance],
				current_balance + 1,
				u64::MAX
			),
			Error::<Test>::AmountSlippage
		);
	})
}

#[test]
fn remove_liquidity_imbalance_when_min_amounts_of_underlying_tokens_not_reached_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e16 as Balance],
			0,
			u64::MAX,
		));

		let max_pool_token_amount_to_be_burned =
			StableAmm::calculate_currency_amount(pool_id, vec![1e18 as Balance, 1e16 as Balance], false).unwrap();

		let max_pool_token_amount_to_be_burned_negative_slippage = max_pool_token_amount_to_be_burned * 1001 / 1000;

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			pool_id,
			vec![1e16 as Balance, 2e20 as Balance],
			0,
			u64::MAX,
		));

		assert_noop!(
			StableAmm::remove_liquidity_imbalance(
				Origin::signed(BOB),
				pool_id,
				vec![1e18 as Balance, 1e16 as Balance],
				max_pool_token_amount_to_be_burned_negative_slippage,
				u64::MAX
			),
			Error::<Test>::AmountSlippage
		);
	})
}

#[test]
fn remove_liquidity_imbalance_with_expired_deadline_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, lp_currency_id) = setup_test_pool();
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e16 as Balance],
			0,
			u64::MAX,
		));
		let current_balance = <Test as Config>::MultiCurrency::free_balance(lp_currency_id, &BOB);
		System::set_block_number(100);

		assert_noop!(
			StableAmm::remove_liquidity_imbalance(
				Origin::signed(BOB),
				pool_id,
				vec![1e18 as Balance, 1e16 as Balance],
				current_balance,
				99
			),
			Error::<Test>::Deadline
		);
	})
}

#[test]
fn remove_liquidity_one_currency_with_currency_index_out_range_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::calculate_remove_liquidity_one_token(&pool, 1, 5), None);
	})
}

#[test]
fn remove_liquidity_one_currency_calculation_should_work() {
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
		assert_eq!(
			StableAmm::calculate_remove_liquidity_one_token(&pool, 2 * pool_token_balance, 0)
				.unwrap()
				.0,
			2999998601797183633
		);
	})
}

#[test]
fn remove_liquidity_one_currency_calculated_amount_as_min_amount_should_work() {
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
		let calculated_first_token_amount =
			StableAmm::calculate_remove_liquidity_one_token(&pool, pool_token_balance, 0).unwrap();
		assert_eq!(calculated_first_token_amount.0, 2008990034631583696);

		let before = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);

		assert_ok!(StableAmm::remove_liquidity_one_currency(
			Origin::signed(BOB),
			pool_id,
			pool_token_balance,
			0,
			calculated_first_token_amount.0,
			u64::MAX
		));

		let after = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);
		assert_eq!(after - before, 2008990034631583696);
	})
}

#[test]
fn remove_liquidity_one_currency_with_lp_token_amount_exceed_own_should_work() {
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
			StableAmm::remove_liquidity_one_currency(
				Origin::signed(BOB),
				pool_id,
				pool_token_balance + 1,
				0,
				0,
				u64::MAX
			),
			Error::<Test>::InsufficientSupply
		);
	})
}

#[test]
fn remove_liquidity_one_currency_with_min_amount_not_reached_due_to_front_running_should_not_work() {
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
		let calculated_first_token_amount =
			StableAmm::calculate_remove_liquidity_one_token(&pool, pool_token_balance, 0).unwrap();
		assert_eq!(calculated_first_token_amount.0, 2008990034631583696);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			pool_id,
			vec![1e16 as Balance, 1e20 as Balance],
			0,
			u64::MAX,
		));

		assert_noop!(
			StableAmm::remove_liquidity_one_currency(
				Origin::signed(BOB),
				pool_id,
				pool_token_balance,
				0,
				calculated_first_token_amount.0,
				u64::MAX
			),
			Error::<Test>::AmountSlippage
		);
	})
}

#[test]
fn remove_liquidity_one_currency_with_expired_deadline_should_not_work() {
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
			StableAmm::remove_liquidity_one_currency(Origin::signed(BOB), pool_id, pool_token_balance, 0, 0, 99),
			Error::<Test>::Deadline
		);
	})
}

#[test]
fn swap_with_currency_index_out_of_index_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::calculate_swap_amount(&pool, 0, 9, 1e17 as Balance), None);
	})
}

#[test]
fn swap_with_currency_amount_exceed_own_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_noop!(
			StableAmm::swap(Origin::signed(BOB), pool_id, 0, 1, Balance::MAX, 0, u64::MAX),
			Error::<Test>::InsufficientReserve
		);
	})
}

#[test]
fn swap_with_expected_amounts_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();

		let calculated_swap_return = StableAmm::calculate_swap_amount(&pool, 0, 1, 1e17 as Balance).unwrap();
		assert_eq!(calculated_swap_return, 99702611562565289);

		let token_from_balance_before = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);
		let token_to_balance_before = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &BOB);

		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			0,
			1,
			1e17 as Balance,
			calculated_swap_return,
			u64::MAX
		));
		let token_from_balance_after = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);
		let token_to_balance_after = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &BOB);

		assert_eq!(token_from_balance_before - token_from_balance_after, 1e17 as Balance);
		assert_eq!(token_to_balance_after - token_to_balance_before, calculated_swap_return);
	})
}

#[test]
fn swap_when_min_amount_receive_not_reached_due_to_front_running_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		let calculated_swap_return = StableAmm::calculate_swap_amount(&pool, 0, 1, 1e17 as Balance).unwrap();
		assert_eq!(calculated_swap_return, 99702611562565289);

		assert_ok!(StableAmm::swap(
			Origin::signed(CHARLIE),
			pool_id,
			0,
			1,
			1e17 as Balance,
			calculated_swap_return,
			u64::MAX
		));

		assert_noop!(
			StableAmm::swap(
				Origin::signed(BOB),
				pool_id,
				0,
				1,
				1e17 as Balance,
				calculated_swap_return,
				u64::MAX
			),
			Error::<Test>::AmountSlippage
		);
	})
}

#[test]
fn swap_with_lower_min_dy_when_transaction_is_front_ran_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();

		let token_from_balance_before = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);
		let token_to_balance_before = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &BOB);

		// BOB calculates how much token to receive with 1% slippage
		let calculated_swap_return = StableAmm::calculate_swap_amount(&pool, 0, 1, 1e17 as Balance).unwrap();
		assert_eq!(calculated_swap_return, 99702611562565289);
		let calculated_swap_return_with_negative_slippage = calculated_swap_return * 99 / 100;

		// CHARLIE swaps before User 1 does
		assert_ok!(StableAmm::swap(
			Origin::signed(CHARLIE),
			pool_id,
			0,
			1,
			1e17 as Balance,
			0,
			u64::MAX
		));

		// BOB swap with slippage
		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			0,
			1,
			1e17 as Balance,
			calculated_swap_return_with_negative_slippage,
			u64::MAX
		));

		let token_from_balance_after = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);
		let token_to_balance_after = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &BOB);

		assert_eq!(token_from_balance_before - token_from_balance_after, 1e17 as Balance);

		let actual_received_amount = token_to_balance_after - token_to_balance_before;
		assert_eq!(actual_received_amount, 99286252365528551);
		assert!(actual_received_amount > calculated_swap_return_with_negative_slippage);
		assert!(actual_received_amount < calculated_swap_return);
	})
}

#[test]
fn swap_with_expired_deadline_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		System::set_block_number(100);

		assert_noop!(
			StableAmm::swap(Origin::signed(BOB), pool_id, 0, 1, 1e17 as Balance, 0, 99),
			Error::<Test>::Deadline
		);
	})
}

#[test]
fn calculate_virtual_price_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1e18 as Balance));
	})
}

#[test]
fn calculate_virtual_price_after_swap_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			0,
			1,
			1e17 as Balance,
			0,
			u64::MAX
		));
		assert_eq!(
			StableAmm::calculate_virtual_price(pool_id),
			Some(1000050005862349911 as Balance)
		);

		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			1,
			0,
			1e17 as Balance,
			0,
			u64::MAX
		));

		assert_eq!(
			StableAmm::calculate_virtual_price(pool_id),
			Some(1000100104768517937 as Balance)
		);
	})
}

#[test]
fn calculate_virtual_price_after_imbalanced_withdrawal_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 1e18 as Balance],
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			pool_id,
			vec![1e18 as Balance, 1e18 as Balance],
			0,
			u64::MAX,
		));

		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1e18 as Balance));

		assert_ok!(StableAmm::remove_liquidity_imbalance(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 0],
			2e18 as Balance,
			u64::MAX
		));

		assert_eq!(
			StableAmm::calculate_virtual_price(pool_id),
			Some(1000100094088440633 as Balance)
		);

		assert_ok!(StableAmm::remove_liquidity_imbalance(
			Origin::signed(CHARLIE),
			pool_id,
			vec![0, 1e18 as Balance],
			2e18 as Balance,
			u64::MAX
		));
		assert_eq!(
			StableAmm::calculate_virtual_price(pool_id),
			Some(1000200154928939884 as Balance)
		);
	})
}

#[test]
fn calculate_virtual_price_value_unchanged_after_deposits_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		// pool is 1:1 ratio
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1e18 as Balance));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			pool_id,
			vec![1e18 as Balance, 1e18 as Balance],
			0,
			u64::MAX,
		));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1e18 as Balance));

		// pool change 2:1 ratio, virtual_price also change
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			pool_id,
			vec![2e18 as Balance, 0],
			0,
			u64::MAX,
		));
		assert_eq!(
			StableAmm::calculate_virtual_price(pool_id),
			Some(1000167146429977312 as Balance)
		);

		// keep 2:1 ratio after deposit, virtual not change.
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![2e18 as Balance, 1e18 as Balance],
			0,
			u64::MAX,
		));
		assert_eq!(
			StableAmm::calculate_virtual_price(pool_id),
			Some(1000167146429977312 as Balance)
		);
	})
}

#[test]
fn calculate_virtual_price_value_not_change_after_balanced_withdrawal_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 1e18 as Balance],
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::remove_liquidity(
			Origin::signed(BOB),
			pool_id,
			1e18 as Balance,
			vec![0, 0],
			u64::MAX
		));

		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1e18 as Balance));
	})
}

#[test]
fn set_fee_with_non_owner_account_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_noop!(StableAmm::set_fee(Origin::signed(BOB), pool_id, 0, 0,), BadOrigin);

		assert_noop!(
			StableAmm::set_fee(Origin::signed(CHARLIE), pool_id, 1e18 as Balance, 0,),
			BadOrigin
		);
	})
}

#[test]
fn set_fee_with_exceed_threshold_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		assert_noop!(
			StableAmm::set_fee(Origin::root(), pool_id, (1e8 as Balance) + 1, 0,),
			Error::<Test>::ExceedThreshold
		);
	})
}

#[test]
fn set_fee_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		assert_ok!(StableAmm::set_fee(Origin::root(), pool_id, 1e8 as Balance, 0,));

		let pool = StableAmm::pools(pool_id).unwrap();

		assert_eq!(pool.fee, 1e8 as Balance);
	})
}

#[test]
fn set_admin_fee_with_non_owner_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		assert_noop!(
			StableAmm::set_fee(Origin::signed(BOB), pool_id, 1e7 as Balance, 0,),
			BadOrigin
		);
		assert_noop!(
			StableAmm::set_fee(Origin::signed(CHARLIE), pool_id, 1e7 as Balance, 1e10 as Balance,),
			BadOrigin
		);
	})
}

#[test]
fn set_admin_fee_with_exceed_threshold_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		assert_noop!(
			StableAmm::set_fee(Origin::root(), pool_id, 1e7 as Balance, (1e10 as Balance) + 1,),
			Error::<Test>::ExceedThreshold
		);
	})
}

#[test]
fn set_admin_fee_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		assert_ok!(StableAmm::set_fee(
			Origin::root(),
			pool_id,
			1e7 as Balance,
			1e10 as Balance,
		));

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(pool.admin_fee, 1e10 as Balance);
	})
}

#[test]
fn get_admin_balance_with_index_out_of_range_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		assert_eq!(StableAmm::get_admin_balance(pool_id, 3), None);
	})
}

#[test]
fn get_admin_balance_always_zero_when_admin_fee_equal_zero() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_eq!(StableAmm::get_admin_balance(pool_id, 0), Some(Zero::zero()));
		assert_eq!(StableAmm::get_admin_balance(pool_id, 1), Some(Zero::zero()));

		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			0,
			1,
			1e17 as Balance,
			0,
			u64::MAX
		));

		assert_eq!(StableAmm::get_admin_balance(pool_id, 0), Some(Zero::zero()));
		assert_eq!(StableAmm::get_admin_balance(pool_id, 1), Some(Zero::zero()));
	})
}

#[test]
fn get_admin_balance_with_expected_amount_after_swap_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_ok!(StableAmm::set_fee(
			Origin::root(),
			pool_id,
			1e7 as Balance,
			1e8 as Balance,
		));

		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			0,
			1,
			1e17 as Balance,
			0,
			u64::MAX
		));
		assert_eq!(StableAmm::get_admin_balance(pool_id, 0), Some(Zero::zero()));
		assert_eq!(StableAmm::get_admin_balance(pool_id, 1), Some(998024139765));

		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			1,
			0,
			1e17 as Balance,
			0,
			u64::MAX
		));

		assert_eq!(StableAmm::get_admin_balance(pool_id, 0), Some(1001973776101));
		assert_eq!(StableAmm::get_admin_balance(pool_id, 1), Some(998024139765));
	})
}

#[test]
fn withdraw_admin_fee_with_non_owner_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		assert_noop!(StableAmm::withdraw_admin_fee(Origin::signed(BOB), pool_id), BadOrigin);
		assert_noop!(
			StableAmm::withdraw_admin_fee(Origin::signed(CHARLIE), pool_id),
			BadOrigin
		);
	})
}

#[test]
fn withdraw_admin_fee_when_no_admin_fee_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		assert_ok!(StableAmm::set_fee(
			Origin::root(),
			pool_id,
			1e7 as Balance,
			1e8 as Balance
		));

		let first_token_balance_before =
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &pool.admin_fee_receiver);
		let second_token_balance_before =
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &pool.admin_fee_receiver);

		assert_ok!(StableAmm::withdraw_admin_fee(Origin::root(), pool_id));

		let first_token_balance_after =
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &pool.admin_fee_receiver);
		let second_token_balance_after =
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &pool.admin_fee_receiver);

		assert_eq!(first_token_balance_before, first_token_balance_after);
		assert_eq!(second_token_balance_before, second_token_balance_after);
	})
}

#[test]
fn withdraw_admin_fee_with_expected_amount_of_fees_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();

		assert_ok!(StableAmm::set_fee(
			Origin::root(),
			pool_id,
			1e7 as Balance,
			1e8 as Balance,
		));

		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			0,
			1,
			1e17 as Balance,
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			1,
			0,
			1e17 as Balance,
			0,
			u64::MAX
		));

		assert_eq!(StableAmm::get_admin_balance(pool_id, 0), Some(1001973776101));
		assert_eq!(StableAmm::get_admin_balance(pool_id, 1), Some(998024139765));

		let first_token_balance_before =
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &pool.admin_fee_receiver);
		let second_token_balance_before =
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &pool.admin_fee_receiver);

		assert_ok!(StableAmm::withdraw_admin_fee(Origin::root(), pool_id));

		let first_token_balance_after =
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &pool.admin_fee_receiver);
		let second_token_balance_after =
			<Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &pool.admin_fee_receiver);

		assert_eq!(first_token_balance_after - first_token_balance_before, 1001973776101);
		assert_eq!(second_token_balance_after - second_token_balance_before, 998024139765);
	})
}

#[test]
fn withdraw_admin_fee_has_no_impact_on_user_withdrawal() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();

		assert_ok!(StableAmm::set_fee(
			Origin::root(),
			pool_id,
			1e7 as Balance,
			1e8 as Balance,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 1e18 as Balance],
			0,
			u64::MAX
		));

		for _i in 0..10 {
			assert_ok!(StableAmm::swap(
				Origin::signed(CHARLIE),
				pool_id,
				0,
				1,
				1e17 as Balance,
				0,
				u64::MAX
			));

			assert_ok!(StableAmm::swap(
				Origin::signed(CHARLIE),
				pool_id,
				1,
				0,
				1e17 as Balance,
				0,
				u64::MAX
			));
		}

		assert_ok!(StableAmm::withdraw_admin_fee(Origin::root(), pool_id));

		let first_token_balance_before = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);
		let second_token_balance_before = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &BOB);

		let pool_token_balance = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);

		assert_ok!(StableAmm::withdraw_admin_fee(Origin::root(), pool_id));

		assert_ok!(StableAmm::remove_liquidity(
			Origin::signed(BOB),
			pool_id,
			pool_token_balance,
			vec![0, 0],
			u64::MAX,
		));

		let first_token_balance_after = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN1_SYMBOL), &BOB);
		let second_token_balance_after = <Test as Config>::MultiCurrency::free_balance(Token(TOKEN2_SYMBOL), &BOB);

		assert_eq!(
			first_token_balance_after - first_token_balance_before,
			1000009516257264879
		);
		assert_eq!(
			second_token_balance_after - second_token_balance_before,
			1000980987206499309
		);
	})
}

#[test]
fn ramp_a_upwards_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		mine_block();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 0],
			0,
			u64::MAX
		));

		mine_block();

		let end_timestamp = Timestamp::now() / 1000 + 14 * DAYS + 1;
		assert_ok!(StableAmm::ramp_a(Origin::root(), pool_id, 100, end_timestamp.into()));

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5000));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1000167146429977312));

		mine_block_with_timestamp(Timestamp::now() / 1000 + 100000);

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5413));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1000258443200231295));

		mine_block_with_timestamp(end_timestamp.into());
		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(10000));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1000771363829405068));
	})
}

#[test]
fn ramp_a_downward_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		mine_block();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 0],
			0,
			u64::MAX
		));

		mine_block();

		let end_timestamp = Timestamp::now() / 1000 + 14 * DAYS + 1;
		assert_ok!(StableAmm::ramp_a(Origin::root(), pool_id, 25, end_timestamp.into()));

		let pool = StableAmm::pools(pool_id).unwrap();

		assert_eq!(StableAmm::get_a_precise(&pool), Some(5000));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1000167146429977312));

		mine_block_with_timestamp(Timestamp::now() / 1000 + 100000);

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(4794));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1000115870150391894));

		mine_block_with_timestamp(end_timestamp);
		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(2500));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(998999574522335473));
	})
}

#[test]
fn ramp_a_with_non_owner_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();

		mine_block();
		let end_timestamp = Timestamp::now() / 1000 + 14 * DAYS + 1;

		assert_noop!(
			StableAmm::ramp_a(Origin::signed(BOB), pool_id, 55, end_timestamp.into()),
			BadOrigin
		);
	})
}

#[test]
fn ramp_a_not_delay_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		mine_block();

		let end_timestamp = Timestamp::now() / 1000 + 14 * DAYS + 1;
		assert_ok!(StableAmm::ramp_a(Origin::root(), pool_id, 55, end_timestamp.into()));

		assert_noop!(
			StableAmm::ramp_a(Origin::root(), pool_id, 55, end_timestamp.into()),
			Error::<Test>::RampADelay
		);
	})
}

#[test]
fn ramp_a_out_of_range_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		mine_block();

		let end_timestamp = Timestamp::now() / 1000 + 14 * DAYS + 1;

		assert_noop!(
			StableAmm::ramp_a(Origin::root(), pool_id, 0, end_timestamp.into()),
			Error::<Test>::ExceedThreshold
		);

		assert_noop!(
			StableAmm::ramp_a(Origin::root(), pool_id, 501, end_timestamp.into()),
			Error::<Test>::ExceedMaxAChange
		);
	})
}

#[test]
fn stop_ramp_a_should_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		mine_block();

		let end_timestamp = Timestamp::now() / 1000 + 14 * DAYS + 100;
		assert_ok!(StableAmm::ramp_a(Origin::root(), pool_id, 100, end_timestamp.into()));

		mine_block_with_timestamp(Timestamp::now() / 1000 + 100000);

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5413));

		assert_ok!(StableAmm::stop_ramp_a(Origin::root(), pool_id));
		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5413));

		mine_block_with_timestamp(end_timestamp);
		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5413));
	})
}

#[test]
fn stop_ramp_a_repeat_should_not_work() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		mine_block();

		let end_timestamp = Timestamp::now() / 1000 + 14 * DAYS + 100;
		assert_ok!(StableAmm::ramp_a(Origin::root(), pool_id, 100, end_timestamp.into()));

		mine_block_with_timestamp(Timestamp::now() / 1000 + 100000);

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5413));

		assert_ok!(StableAmm::stop_ramp_a(Origin::root(), pool_id));
		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5413));

		assert_noop!(
			StableAmm::stop_ramp_a(Origin::root(), pool_id),
			Error::<Test>::AlreadyStoppedRampA
		);
	})
}

#[test]
fn check_maximum_differences_in_a_and_virtual_price_when_time_manipulations_and_increasing_a() {
	new_test_ext().execute_with(|| {
		mine_block();

		let (pool_id, _) = setup_test_pool();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(ALICE),
			pool_id,
			vec![1e18 as Balance, 0],
			0,
			u64::MAX,
		));

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5000));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1000167146429977312));

		let end_timestamp = Timestamp::now() / 1000 + 14 * DAYS + 100;
		assert_ok!(StableAmm::ramp_a(Origin::root(), pool_id, 100, end_timestamp.into()));

		// Malicious miner skips 900 seconds
		set_block_timestamp(Timestamp::now() / 1000 + 900);

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5003));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1000167862696363286));
	})
}

#[test]
fn check_maximum_differences_in_a_and_virtual_price_when_time_manipulations_and_decreasing_a() {
	new_test_ext().execute_with(|| {
		mine_block();

		let (pool_id, _) = setup_test_pool();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(ALICE),
			pool_id,
			vec![1e18 as Balance, 0],
			0,
			u64::MAX,
		));

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5000));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1000167146429977312));

		let end_timestamp = Timestamp::now() / 1000 + 14 * DAYS + 100;

		assert_ok!(StableAmm::ramp_a(Origin::root(), pool_id, 25, end_timestamp.into()));

		// Malicious miner skips 900 seconds
		set_block_timestamp(Timestamp::now() / 1000 + 900);

		let pool = StableAmm::pools(pool_id).unwrap();
		assert_eq!(StableAmm::get_a_precise(&pool), Some(4999));
		assert_eq!(StableAmm::calculate_virtual_price(pool_id), Some(1000166907487883089));
	})
}

struct AttackContext {
	pub initial_attacker_balances: Vec<Balance>,
	pub initial_pool_balances: Vec<Balance>,
	pub pool_currencies: Vec<CurrencyId>,
	pub attacker: AccountId,
	pub pool_id: PoolId,
}

fn prepare_attack_context(new_a: Balance) -> AttackContext {
	mine_block();

	let (pool_id, _) = setup_test_pool();
	let attacker = BOB;
	let pool = StableAmm::pools(pool_id).unwrap();

	let mut attack_balances = Vec::new();
	for currency_id in pool.currency_ids.iter() {
		attack_balances.push(<Test as Config>::MultiCurrency::free_balance(*currency_id, &attacker));
	}

	assert_ok!(StableAmm::ramp_a(
		Origin::root(),
		pool_id,
		new_a,
		(Timestamp::now() / 1000 + 14 * DAYS).into()
	));

	assert_eq!(attack_balances[0], 1e20 as Balance);
	assert_eq!(attack_balances[1], 1e20 as Balance);

	assert_eq!(pool.balances[0], 1e18 as Balance);
	assert_eq!(pool.balances[1], 1e18 as Balance);

	AttackContext {
		initial_attacker_balances: attack_balances,
		initial_pool_balances: pool.balances.clone(),
		pool_currencies: pool.currency_ids.clone(),
		attacker,
		pool_id,
	}
}

#[test]
fn check_when_ramp_a_upwards_and_tokens_price_equally() {
	new_test_ext().execute_with(|| {
		let context = prepare_attack_context(100);

		// Swap 1e18 of firstToken to secondToken, causing massive imbalance in the pool
		assert_ok!(StableAmm::swap(
			Origin::signed(context.attacker),
			context.pool_id,
			0,
			1,
			1e18 as Balance,
			0,
			u64::MAX
		));
		let second_token_output =
			<Test as Config>::MultiCurrency::free_balance(context.pool_currencies[1], &context.attacker)
				- context.initial_attacker_balances[1];

		assert_eq!(second_token_output, 908591742545002306);

		// Pool is imbalanced! Now trades from secondToken -> firstToken may be profitable in small sizes
		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert_eq!(pool.balances[0], 2e18 as Balance);
		assert_eq!(pool.balances[1], 91408257454997694);

		// Malicious miner skips 900 seconds
		set_block_timestamp(Timestamp::now() / 1000 + 900);

		assert_eq!(StableAmm::get_a_precise(&pool), Some(5003));

		let balances_before = get_user_token_balances(&context.pool_currencies, &context.attacker);

		assert_ok!(StableAmm::swap(
			Origin::signed(context.attacker),
			context.pool_id,
			1,
			0,
			second_token_output,
			0,
			u64::MAX
		));

		let first_token_output =
			<Test as Config>::MultiCurrency::free_balance(context.pool_currencies[0], &context.attacker)
				- balances_before[0];
		assert_eq!(first_token_output, 997214696574405737);

		let final_attacker_balances = get_user_token_balances(&context.pool_currencies, &context.attacker);

		assert!(final_attacker_balances[0] < context.initial_attacker_balances[0]);
		assert_eq!(final_attacker_balances[1], context.initial_attacker_balances[1]);
		assert_eq!(
			context.initial_attacker_balances[0] - final_attacker_balances[0],
			2785303425594263
		);
		assert_eq!(context.initial_attacker_balances[1] - final_attacker_balances[1], 0);

		// checked pool balance,
		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert!(pool.balances[0] > context.initial_pool_balances[0]);
		assert_eq!(pool.balances[1], context.initial_pool_balances[1]);

		assert_eq!(pool.balances[0] - context.initial_pool_balances[0], 2785303425594263);
		assert_eq!(pool.balances[1] - context.initial_pool_balances[1], 0);
	})
}

#[test]
fn check_when_ramp_a_upwards_and_tokens_price_unequally() {
	new_test_ext().execute_with(|| {
		let mut context = prepare_attack_context(100);

		// Set up pool to be imbalanced prior to the attack
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(ALICE),
			context.pool_id,
			vec![0, 2e18 as Balance],
			0,
			u64::MAX,
		));

		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert_eq!(pool.balances[0], 1e18 as Balance);
		assert_eq!(pool.balances[1], 3e18 as Balance);

		// rewrite pool balances
		context.initial_pool_balances = pool.balances.clone();

		// Swap 1e18 of firstToken to secondToken, resolving imbalance in the pool
		assert_ok!(StableAmm::swap(
			Origin::signed(context.attacker),
			context.pool_id,
			0,
			1,
			1e18 as Balance,
			0,
			u64::MAX
		));
		let second_token_output =
			<Test as Config>::MultiCurrency::free_balance(context.pool_currencies[1], &context.attacker)
				- context.initial_attacker_balances[1];

		assert_eq!(second_token_output, 1011933251060681353);

		// Pool is imbalanced! Now trades from secondToken -> firstToken may be profitable in small sizes
		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert_eq!(pool.balances[0], 2e18 as Balance);
		assert_eq!(pool.balances[1], 1988066748939318647);

		// Malicious miner skips 900 seconds
		set_block_timestamp(Timestamp::now() / 1000 + 900);
		assert_eq!(StableAmm::get_a_precise(&pool), Some(5003));

		let balances_before = get_user_token_balances(&context.pool_currencies, &context.attacker);

		assert_ok!(StableAmm::swap(
			Origin::signed(context.attacker),
			context.pool_id,
			1,
			0,
			second_token_output,
			0,
			u64::MAX
		));

		let first_token_output =
			<Test as Config>::MultiCurrency::free_balance(context.pool_currencies[0], &context.attacker)
				- balances_before[0];
		assert_eq!(first_token_output, 998017518949630644);

		let final_attacker_balances = get_user_token_balances(&context.pool_currencies, &context.attacker);

		assert!(final_attacker_balances[0] < context.initial_attacker_balances[0]);
		assert_eq!(final_attacker_balances[1], context.initial_attacker_balances[1]);
		assert_eq!(
			context.initial_attacker_balances[0] - final_attacker_balances[0],
			1982481050369356
		);
		assert_eq!(context.initial_attacker_balances[1] - final_attacker_balances[1], 0);

		// checked pool balance,
		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert!(pool.balances[0] > context.initial_pool_balances[0]);
		assert_eq!(pool.balances[1], context.initial_pool_balances[1]);

		assert_eq!(pool.balances[0] - context.initial_pool_balances[0], 1982481050369356);
		assert_eq!(pool.balances[1] - context.initial_pool_balances[1], 0);
	})
}

#[test]
fn check_when_ramp_a_downwards_and_tokens_price_equally() {
	new_test_ext().execute_with(|| {
		let context = prepare_attack_context(25);
		// Swap 1e18 of firstToken to secondToken, causing massive imbalance in the pool
		assert_ok!(StableAmm::swap(
			Origin::signed(context.attacker),
			context.pool_id,
			0,
			1,
			1e18 as Balance,
			0,
			u64::MAX
		));
		let second_token_output =
			<Test as Config>::MultiCurrency::free_balance(context.pool_currencies[1], &context.attacker)
				- context.initial_attacker_balances[1];

		assert_eq!(second_token_output, 908591742545002306);

		// Pool is imbalanced! Now trades from secondToken -> firstToken may be profitable in small sizes
		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert_eq!(pool.balances[0], 2e18 as Balance);
		assert_eq!(pool.balances[1], 91408257454997694);

		// Malicious miner skips 900 seconds
		set_block_timestamp(Timestamp::now() / 1000 + 900);

		assert_eq!(StableAmm::get_a_precise(&pool), Some(4999));

		let balances_before = get_user_token_balances(&context.pool_currencies, &context.attacker);

		assert_ok!(StableAmm::swap(
			Origin::signed(context.attacker),
			context.pool_id,
			1,
			0,
			second_token_output,
			0,
			u64::MAX
		));

		let first_token_output =
			<Test as Config>::MultiCurrency::free_balance(context.pool_currencies[0], &context.attacker)
				- balances_before[0];
		assert_eq!(first_token_output, 997276754500361021);

		let final_attacker_balances = get_user_token_balances(&context.pool_currencies, &context.attacker);

		assert!(final_attacker_balances[0] < context.initial_attacker_balances[0]);
		assert_eq!(final_attacker_balances[1], context.initial_attacker_balances[1]);
		assert_eq!(
			context.initial_attacker_balances[0] - final_attacker_balances[0],
			2723245499638979
		);
		assert_eq!(context.initial_attacker_balances[1] - final_attacker_balances[1], 0);

		// checked pool balance,
		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert!(pool.balances[0] > context.initial_pool_balances[0]);
		assert_eq!(pool.balances[1], context.initial_pool_balances[1]);

		assert_eq!(pool.balances[0] - context.initial_pool_balances[0], 2723245499638979);
		assert_eq!(pool.balances[1] - context.initial_pool_balances[1], 0);
	})
}

#[test]
fn check_when_ramp_a_downwards_and_tokens_price_unequally() {
	new_test_ext().execute_with(|| {
		let mut context = prepare_attack_context(25);

		// Set up pool to be imbalanced prior to the attack
		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(ALICE),
			context.pool_id,
			vec![0, 2e18 as Balance],
			0,
			u64::MAX,
		));

		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert_eq!(pool.balances[0], 1e18 as Balance);
		assert_eq!(pool.balances[1], 3e18 as Balance);

		// rewrite pool balances
		context.initial_pool_balances = pool.balances.clone();

		// Swap 1e18 of firstToken to secondToken, resolving imbalance in the pool
		assert_ok!(StableAmm::swap(
			Origin::signed(context.attacker),
			context.pool_id,
			0,
			1,
			1e18 as Balance,
			0,
			u64::MAX
		));
		let second_token_output =
			<Test as Config>::MultiCurrency::free_balance(context.pool_currencies[1], &context.attacker)
				- context.initial_attacker_balances[1];

		assert_eq!(second_token_output, 1011933251060681353);

		// Pool is imbalanced! Now trades from secondToken -> firstToken may be profitable in small sizes
		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert_eq!(pool.balances[0], 2e18 as Balance);
		assert_eq!(pool.balances[1], 1988066748939318647);

		// Malicious miner skips 900 seconds
		set_block_timestamp(Timestamp::now() / 1000 + 900);
		assert_eq!(StableAmm::get_a_precise(&pool), Some(4999));

		let balances_before = get_user_token_balances(&context.pool_currencies, &context.attacker);

		assert_ok!(StableAmm::swap(
			Origin::signed(context.attacker),
			context.pool_id,
			1,
			0,
			second_token_output,
			0,
			u64::MAX
		));

		let first_token_output =
			<Test as Config>::MultiCurrency::free_balance(context.pool_currencies[0], &context.attacker)
				- balances_before[0];
		assert_eq!(first_token_output, 998007711333645455);

		let final_attacker_balances = get_user_token_balances(&context.pool_currencies, &context.attacker);

		assert!(final_attacker_balances[0] < context.initial_attacker_balances[0]);
		assert_eq!(final_attacker_balances[1], context.initial_attacker_balances[1]);
		assert_eq!(
			context.initial_attacker_balances[0] - final_attacker_balances[0],
			1992288666354545
		);
		assert_eq!(context.initial_attacker_balances[1] - final_attacker_balances[1], 0);

		// checked pool balance,
		let pool = StableAmm::pools(context.pool_id).unwrap();
		assert!(pool.balances[0] > context.initial_pool_balances[0]);
		assert_eq!(pool.balances[1], context.initial_pool_balances[1]);

		assert_eq!(pool.balances[0] - context.initial_pool_balances[0], 1992288666354545);
		assert_eq!(pool.balances[1] - context.initial_pool_balances[1], 0);
	})
}

fn mint_more_currencies(accounts: Vec<AccountId>, currencies: Vec<CurrencyId>, balances: Vec<Balance>) {
	assert_eq!(currencies.len(), balances.len());
	for account in accounts.iter() {
		for (i, currency_id) in currencies.iter().enumerate() {
			assert_ok!(Tokens::set_balance(
				Origin::root(),
				*account,
				*currency_id,
				balances[i],
				0,
			));
		}
	}
}

#[test]
fn check_arithmetic_in_add_liquidity_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();

		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids,
			vec![1_000_000_000e18 as Balance, 1_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &ALICE),
			2e18 as Balance
		);
		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB),
			299583613596961209609933
		);
		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE),
			398605324970970482408685465
		);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		assert_eq!(
			<Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB),
			401292897411939364910247311
		);
	})
}

#[test]
fn check_arithmetic_in_remove_liquidity_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();

		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids.clone(),
			vec![10_000_000_000e18 as Balance, 10_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		let user1_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user1_token0_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &BOB);
		let user1_token1_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &BOB);

		assert_ok!(StableAmm::remove_liquidity(
			Origin::signed(BOB),
			pool_id,
			user1_pool_lp_balance_before,
			vec![0, 0],
			u64::MAX
		));

		let user1_pool_lp_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user1_token0_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &BOB);
		let user1_token1_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &BOB);

		assert_eq!(user1_pool_lp_balance_after, 0);
		assert_eq!(
			user1_token0_balance_after - user1_token0_balance_before,
			200722146595179027183639390
		);
		assert_eq!(
			user1_token1_balance_after - user1_token1_balance_before,
			200772314589703770768227294
		);

		// user2 remove liquidity
		let user2_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);
		let user2_token0_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &CHARLIE);
		let user2_token1_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &CHARLIE);

		assert_ok!(StableAmm::remove_liquidity(
			Origin::signed(CHARLIE),
			pool_id,
			user2_pool_lp_balance_before,
			vec![0, 0],
			u64::MAX
		));

		let user2_pool_lp_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);
		let user2_token0_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &CHARLIE);
		let user2_token1_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &CHARLIE);

		assert_eq!(user2_pool_lp_balance_after, 0);
		assert_eq!(
			user2_token0_balance_after - user2_token0_balance_before,
			199377853404443702798886871
		);
		assert_eq!(
			user2_token1_balance_after - user2_token1_balance_before,
			199427685409668927405371910
		);
	})
}

#[test]
fn check_arithmetic_in_remove_liquidity_one_currency_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids.clone(),
			vec![10_000_000_000e18 as Balance, 10_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		let user1_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user1_token0_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &BOB);
		let user1_token1_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &BOB);

		let user2_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);
		let user2_token0_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &CHARLIE);
		let user2_token1_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &CHARLIE);

		assert_ok!(StableAmm::remove_liquidity_one_currency(
			Origin::signed(BOB),
			pool_id,
			user1_pool_lp_balance_before,
			0,
			0,
			u64::MAX
		));

		let user1_pool_lp_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user1_token0_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &BOB);
		let user1_token1_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &BOB);

		assert_eq!(user1_pool_lp_balance_after, 0);
		assert_eq!(
			user1_token0_balance_after - user1_token0_balance_before,
			382567648485687465509067831
		);
		assert_eq!(user1_token1_balance_after - user1_token1_balance_before, 0);

		assert_ok!(StableAmm::remove_liquidity_one_currency(
			Origin::signed(CHARLIE),
			pool_id,
			user2_pool_lp_balance_before,
			0,
			0,
			u64::MAX
		));

		let user2_pool_lp_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);
		let user2_token0_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &CHARLIE);
		let user2_token1_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &CHARLIE);

		assert_eq!(user2_pool_lp_balance_after, 0);
		assert_eq!(
			user2_token0_balance_after - user2_token0_balance_before,
			17532352514268550250562021
		);
		assert_eq!(user2_token1_balance_after - user2_token1_balance_before, 0);
	})
}

#[test]
fn check_arithmetic_in_remove_liquidity_imbalance_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids.clone(),
			vec![10_000_000_000e18 as Balance, 10_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		let user1_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user1_token0_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &BOB);
		let user1_token1_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &BOB);

		let user2_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);
		let user2_token0_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &CHARLIE);
		let user2_token1_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &CHARLIE);

		assert_ok!(StableAmm::remove_liquidity_imbalance(
			Origin::signed(BOB),
			pool_id,
			vec![300000000000000000000000000, 100000000000000000000000000],
			user1_pool_lp_balance_before,
			u64::MAX
		));

		let user1_pool_lp_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user1_token0_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &BOB);
		let user1_token1_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &BOB);

		assert_eq!(
			user1_pool_lp_balance_before - user1_pool_lp_balance_after,
			401193808332107345545123455
		);
		assert_eq!(
			user1_token0_balance_after - user1_token0_balance_before,
			300000000000000000000000000
		);
		assert_eq!(
			user1_token1_balance_after - user1_token1_balance_before,
			100000000000000000000000000
		);

		assert_ok!(StableAmm::remove_liquidity_imbalance(
			Origin::signed(CHARLIE),
			pool_id,
			vec![100000000000000000000000000, 300000000000000000000000000],
			user2_pool_lp_balance_before,
			u64::MAX
		));

		let user2_pool_lp_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);
		let user2_token0_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &CHARLIE);
		let user2_token1_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &CHARLIE);

		assert_eq!(user2_pool_lp_balance_after, 200293563918551832434667);
		assert_eq!(
			user2_token0_balance_after - user2_token0_balance_before,
			100000000000000000000000000
		);
		assert_eq!(
			user2_token1_balance_after - user2_token1_balance_before,
			300000000000000000000000000
		);
	})
}

#[test]
fn check_arithmetic_in_swap_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids.clone(),
			vec![10_000_000_000e18 as Balance, 10_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		let user1_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user1_token0_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &BOB);
		let user1_token1_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &BOB);

		let user2_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);
		let user2_token0_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &CHARLIE);
		let user2_token1_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &CHARLIE);

		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			0,
			1,
			100000000000000000000000000,
			0,
			u64::MAX
		));

		let user1_pool_lp_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user1_token0_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &BOB);
		let user1_token1_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &BOB);

		assert_eq!(user1_pool_lp_balance_before, user1_pool_lp_balance_after);
		assert_eq!(
			user1_token0_balance_before - user1_token0_balance_after,
			100000000000000000000000000
		);
		assert_eq!(
			user1_token1_balance_after - user1_token1_balance_before,
			99382677941655828590709465
		);

		assert_ok!(StableAmm::swap(
			Origin::signed(CHARLIE),
			pool_id,
			1,
			0,
			100000000000000000000000000,
			0,
			u64::MAX
		));

		let user2_pool_lp_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);
		let user2_token0_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &CHARLIE);
		let user2_token1_balance_after = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &CHARLIE);

		assert_eq!(user2_pool_lp_balance_after, user2_pool_lp_balance_before);
		assert_eq!(
			user2_token0_balance_after - user2_token0_balance_before,
			100416682007269587274140452
		);
		assert_eq!(
			user2_token1_balance_before - user2_token1_balance_after,
			100000000000000000000000000
		);

		let pool = StableAmm::pools(pool_id).unwrap();

		// check pool balances
		assert_eq!(pool.balances[0], 399683318992730412725859548);
		assert_eq!(pool.balances[1], 400817323058344171409290535);

		let pool_token0_balance = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[0], &POOL0ACCOUNTID);
		let pool_token1_balance = <Test as Config>::MultiCurrency::free_balance(pool.currency_ids[1], &POOL0ACCOUNTID);
		assert_eq!(pool.balances[0], pool_token0_balance);
		assert_eq!(pool.balances[1], pool_token1_balance);
	})
}

#[test]
fn check_arithmetic_in_add_liquidity_with_admin_fee_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids.clone(),
			vec![10_000_000_000e18 as Balance, 10_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::set_fee(Origin::root(), pool_id, SWAP_FEE, MAX_ADMIN_FEE,));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		let admin_balances = StableAmm::get_admin_balances(pool_id);
		assert_eq!(admin_balances[0], 116218703966498771606127);
		assert_eq!(admin_balances[1], 117921007525488838747514);
	})
}

#[test]
fn check_arithmetic_in_remove_liquidity_with_admin_fee_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids.clone(),
			vec![10_000_000_000e18 as Balance, 10_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::set_fee(Origin::root(), pool_id, SWAP_FEE, MAX_ADMIN_FEE,));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		let user1_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);

		assert_ok!(StableAmm::remove_liquidity(
			Origin::signed(BOB),
			pool_id,
			user1_pool_lp_balance_before,
			vec![0, 0],
			u64::MAX
		));
		// user2 remove liquidity
		let user2_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);

		assert_ok!(StableAmm::remove_liquidity(
			Origin::signed(CHARLIE),
			pool_id,
			user2_pool_lp_balance_before,
			vec![0, 0],
			u64::MAX
		));

		let admin_balances = StableAmm::get_admin_balances(pool_id);
		assert_eq!(admin_balances[0], 116218703966498771606127);
		assert_eq!(admin_balances[1], 117921007525488838747514);
	})
}

#[test]
fn check_arithmetic_in_remove_liquidity_one_currency_with_admin_fee_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids.clone(),
			vec![10_000_000_000e18 as Balance, 10_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::set_fee(Origin::root(), pool_id, SWAP_FEE, MAX_ADMIN_FEE,));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		let user1_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user2_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);

		assert_ok!(StableAmm::remove_liquidity_one_currency(
			Origin::signed(BOB),
			pool_id,
			user1_pool_lp_balance_before,
			0,
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::remove_liquidity_one_currency(
			Origin::signed(CHARLIE),
			pool_id,
			user2_pool_lp_balance_before,
			0,
			0,
			u64::MAX
		));

		let admin_balances = StableAmm::get_admin_balances(pool_id);
		assert_eq!(admin_balances[0], 162191894424543883363758);
		assert_eq!(admin_balances[1], 117921007525488838747514);

		let balances = StableAMM::pools(pool_id).unwrap().balances;
		assert_eq!(balances[0], 43828306204680);
		assert_eq!(balances[1], 400082079992474511161252486);
	})
}

#[test]
fn check_arithmetic_in_remove_liquidity_imbalance_with_admin_fee_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids.clone(),
			vec![10_000_000_000e18 as Balance, 10_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::set_fee(Origin::root(), pool_id, SWAP_FEE, MAX_ADMIN_FEE,));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		let user1_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);
		let user2_pool_lp_balance_before = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &CHARLIE);

		assert_ok!(StableAmm::remove_liquidity_imbalance(
			Origin::signed(BOB),
			pool_id,
			vec![200000000000000000000000000, 100000000000000000000000000],
			user1_pool_lp_balance_before,
			u64::MAX
		));

		assert_ok!(StableAmm::remove_liquidity_imbalance(
			Origin::signed(CHARLIE),
			pool_id,
			vec![100000000000000000000000000, 200000000000000000000000000],
			user2_pool_lp_balance_before,
			u64::MAX
		));

		let admin_balances = StableAmm::get_admin_balances(pool_id);
		assert_eq!(admin_balances[0], 151146217664745609762144);
		assert_eq!(admin_balances[1], 152991616465138594784072);

		let balances = StableAMM::pools(pool_id).unwrap().balances;
		assert_eq!(balances[0], 99948854782335254390237856);
		assert_eq!(balances[1], 100047009383534861405215928);
	})
}

#[test]
fn check_arithmetic_in_swap_imbalance_with_admin_fee_should_successfully() {
	new_test_ext().execute_with(|| {
		let (pool_id, _) = setup_test_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		mint_more_currencies(
			vec![BOB, CHARLIE],
			pool.currency_ids.clone(),
			vec![10_000_000_000e18 as Balance, 10_000_000_000e18 as Balance],
		);

		assert_ok!(StableAmm::set_fee(Origin::root(), pool_id, SWAP_FEE, MAX_ADMIN_FEE,));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![100000000000000000000000, 200000000000000000000000], // [100_000e18, 200_00018]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(CHARLIE),
			0,
			vec![100000000000000000000000000, 300000000000000000000000000], // [100_000_000e18, 300_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			0,
			vec![300000000000000000000000000, 100000000000000000000000000], // [300_000_000e18, 100_000_000e18]
			0,
			u64::MAX,
		));

		assert_ok!(StableAmm::swap(
			Origin::signed(BOB),
			pool_id,
			0,
			1,
			100000000000000000000000000,
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::swap(
			Origin::signed(CHARLIE),
			pool_id,
			1,
			0,
			100000000000000000000000000,
			0,
			u64::MAX
		));

		let admin_balances = StableAmm::get_admin_balances(pool_id);
		assert_eq!(admin_balances[0], 216736707365806476948490);
		assert_eq!(admin_balances[1], 217402988477687128132247);

		let balances = StableAMM::pools(pool_id).unwrap().balances;
		assert_eq!(balances[0], 399465778896725795886030548);
		assert_eq!(balances[1], 400600099040276221776518758);
	})
}

#[test]
fn add_pool_and_base_pool_liquidity_should_work() {
	new_test_ext().execute_with(|| {
		let (basic_pool_id, pool_id) = setup_test_pool_and_base_pool();

		let expected_mint_amount = StableAmm::calculate_currency_amount(pool_id, vec![3e18 as Balance, 1e6 as Balance], true).unwrap();

		let pool = StableAmm::pools(pool_id).unwrap();

		assert_ok!(StableAmm::add_pool_and_base_pool_liquidity(
			Origin::signed(BOB),
			pool_id,
			basic_pool_id,
			vec![0, 1e6 as Balance],
			vec![1e18 as Balance, 1e18 as Balance, 1e6 as Balance],
			0,
			u64::MAX
		));

		let lp_amount = <Test as Config>::MultiCurrency::free_balance(pool.lp_currency_id, &BOB);

		assert_eq!(lp_amount, expected_mint_amount);

		assert_eq!(lp_amount, 3987053390609794133);
	})
}

#[test]
fn remove_pool_and_base_pool_liquidity_should_work(){
	new_test_ext().execute_with(|| {
		let (basic_pool_id, pool_id) = setup_test_pool_and_base_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		let base_pool = StableAmm::pools(basic_pool_id).unwrap();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			basic_pool_id,
			vec![1e18 as Balance, 1e18 as Balance, 1e6 as Balance],
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 1e6 as Balance],
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::remove_pool_and_base_pool_liquidity(
			Origin::signed(BOB),
			pool_id,
			basic_pool_id,
			1e18 as Balance,
			vec![0, 0],
			vec![0, 0, 0],
			u64::MAX
		));

		let balances_after = get_user_token_balances(&vec![
			Token(TOKEN1_SYMBOL),
			Token(TOKEN2_SYMBOL),
			Token(TOKEN3_SYMBOL),
			Token(TOKEN4_SYMBOL),
			base_pool.lp_currency_id,
			pool.lp_currency_id,
		],&BOB);

		assert_eq!(balances_after[0], 99166666666666666666);
		assert_eq!(balances_after[1], 99166666666666666666);
		assert_eq!(balances_after[2], 99166666);
		assert_eq!(balances_after[3], 99500000);
		assert_eq!(balances_after[4], 2000000000000000000);
		assert_eq!(balances_after[5], 1000000000000000000);
	})
}

#[test]
fn remove_pool_and_base_pool_liquidity_one_currency_should_work(){
	new_test_ext().execute_with(|| {
		let (basic_pool_id, pool_id) = setup_test_pool_and_base_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		let base_pool = StableAmm::pools(basic_pool_id).unwrap();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			basic_pool_id,
			vec![1e18 as Balance, 1e18 as Balance, 1e6 as Balance],
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 1e6 as Balance],
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::remove_pool_and_base_pool_liquidity_one_currency(
			Origin::signed(BOB),
			pool_id,
			basic_pool_id,
			1e18 as Balance,
			0,
			0,
			u64::MAX
		));


		let balances_after = get_user_token_balances(&vec![
			Token(TOKEN1_SYMBOL),
			Token(TOKEN2_SYMBOL),
			Token(TOKEN3_SYMBOL),
			Token(TOKEN4_SYMBOL),
			base_pool.lp_currency_id,
			pool.lp_currency_id,
		],&BOB);

		assert_eq!(balances_after[0], 99915975025371929634);
		assert_eq!(balances_after[1], 99000000000000000000);
		assert_eq!(balances_after[2], 99000000);
		assert_eq!(balances_after[3], 99000000);
		assert_eq!(balances_after[4], 2000000000000000000);
		assert_eq!(balances_after[5], 1000000000000000000);
	})
}

#[test]
fn swap_pool_from_base_should_work(){
	new_test_ext().execute_with(|| {
		let (basic_pool_id, pool_id) = setup_test_pool_and_base_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		let base_pool = StableAmm::pools(basic_pool_id).unwrap();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			basic_pool_id,
			vec![1e18 as Balance, 1e18 as Balance, 1e6 as Balance],
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 1e6 as Balance],
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::swap_pool_from_base(
			Origin::signed(BOB),
			pool_id,
			basic_pool_id,
			0,
			1,
			1e16 as Balance,
			0,
			u64::MAX
		));


		let balances_after = get_user_token_balances(&vec![
			Token(TOKEN1_SYMBOL),
			Token(TOKEN2_SYMBOL),
			Token(TOKEN3_SYMBOL),
			Token(TOKEN4_SYMBOL),
			base_pool.lp_currency_id,
			pool.lp_currency_id,
		],&BOB);

		assert_eq!(balances_after[0], 98990000000000000000);
		assert_eq!(balances_after[1], 99000000000000000000);
		assert_eq!(balances_after[2], 99000000);
		assert_eq!(balances_after[3], 99009982);
		assert_eq!(balances_after[4], 2000000000000000000);
		assert_eq!(balances_after[5], 2000000000000000000);
	})
}

#[test]
fn swap_pool_to_base_should_work(){
	new_test_ext().execute_with(|| {
		let (basic_pool_id, pool_id) = setup_test_pool_and_base_pool();
		let pool = StableAmm::pools(pool_id).unwrap();
		let base_pool = StableAmm::pools(basic_pool_id).unwrap();

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			basic_pool_id,
			vec![1e18 as Balance, 1e18 as Balance, 1e6 as Balance],
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::add_liquidity(
			Origin::signed(BOB),
			pool_id,
			vec![1e18 as Balance, 1e6 as Balance],
			0,
			u64::MAX
		));

		assert_ok!(StableAmm::swap_pool_to_base(
			Origin::signed(BOB),
			pool_id,
			basic_pool_id,
			1,
			0,
			1e6 as Balance,
			0,
			u64::MAX
		));


		let balances_after = get_user_token_balances(&vec![
			Token(TOKEN1_SYMBOL),
			Token(TOKEN2_SYMBOL),
			Token(TOKEN3_SYMBOL),
			Token(TOKEN4_SYMBOL),
			base_pool.lp_currency_id,
			pool.lp_currency_id,
		],&BOB);

		assert_eq!(balances_after[0], 99881980616021312485);
		assert_eq!(balances_after[1], 99000000000000000000);
		assert_eq!(balances_after[2], 99000000);
		assert_eq!(balances_after[3], 98000000);
		assert_eq!(balances_after[4], 2000000000000000000);
		assert_eq!(balances_after[5], 2000000000000000000);
	})
}