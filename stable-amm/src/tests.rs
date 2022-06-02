use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError::BadOrigin;

use super::{
	mock::{CurrencyId::*, PoolToken::Token as pool_token, PoolType::*, *},
	*,
};

const POOL1ACCOUNTED: AccountId = 33865947477506447919519395693;
type MockPool = Pool<CurrencyId, Balance, AccountId>;

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
			(MAX_A - 1).into(),
			(MAX_SWAP_FEE - 1).into(),
			(MAX_ADMIN_FEE - 1).into()
		));

		assert_eq!(StableAmm::next_pool_id(), 1);

		assert_eq!(
			StableAmm::pools(0),
			Some(MockPool {
				pooled_currency_ids: vec![
					Token(TOKEN1_SYMBOL),
					Token(TOKEN2_SYMBOL),
					Token(TOKEN3_SYMBOL),
					Token(TOKEN4_SYMBOL)
				],
				lp_currency_id: BASIC_4_POOL_CURRENCY_ID,
				token_multipliers: vec![
					checked_pow(10, (POOL_TOKEN_COMMON_DECIMALS - TOKEN1_DECIMAL) as usize).unwrap(),
					checked_pow(10, (POOL_TOKEN_COMMON_DECIMALS - TOKEN2_DECIMAL) as usize).unwrap(),
					checked_pow(10, (POOL_TOKEN_COMMON_DECIMALS - TOKEN3_DECIMAL) as usize).unwrap(),
					checked_pow(10, (POOL_TOKEN_COMMON_DECIMALS - TOKEN4_DECIMAL) as usize).unwrap(),
				],
				balances: vec![Zero::zero(); 4],
				fee: (MAX_SWAP_FEE - 1).into(),
				admin_fee: (MAX_ADMIN_FEE - 1).into(),
				initial_a: ((MAX_A - 1) * 100).into(),
				future_a: ((MAX_A - 1) * 100).into(),
				initial_a_time: 0,
				future_a_time: 0,
				pool_account: POOL1ACCOUNTED,
			})
		);

		assert_eq!(StableAmm::lp_currencies(lp_currency_id), Some(0))
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
