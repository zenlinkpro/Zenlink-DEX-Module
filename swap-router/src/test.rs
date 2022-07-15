use frame_support::{assert_noop, assert_ok};

use super::{
	mock::{CurrencyId::*, *},
	StableSwapMode::FromBase,
	*,
};

const INITIAL_A_VALUE: Balance = 50;
const SWAP_FEE: Balance = 1e7 as Balance;
const ADMIN_FEE: Balance = 0;

fn setup_stable_pools() {
	assert_ok!(StableAMM::create_pool(
		Origin::root(),
		vec![Token(TOKEN1_SYMBOL), Token(TOKEN2_SYMBOL), Token(TOKEN3_SYMBOL)],
		vec![TOKEN1_DECIMAL, TOKEN2_DECIMAL, TOKEN3_DECIMAL],
		INITIAL_A_VALUE,
		SWAP_FEE,
		ADMIN_FEE,
		USER1,
		Vec::from("basic_pool_lp"),
	));

	let pool_id = StableAMM::next_pool_id() - 1;
	let first_pool_lp_currency_id = StableAMM::pools(pool_id).unwrap().lp_currency_id;

	assert_ok!(StableAMM::create_pool(
		Origin::root(),
		vec![first_pool_lp_currency_id, Token(TOKEN4_SYMBOL)],
		vec![18, TOKEN4_DECIMAL],
		INITIAL_A_VALUE,
		SWAP_FEE,
		ADMIN_FEE,
		USER1,
		Vec::from("pool_lp"),
	));

	assert_ok!(StableAMM::add_liquidity(
		Origin::signed(USER1),
		0,
		vec![1e18 as Balance, 1e18 as Balance, 1e6 as Balance],
		0,
		USER1,
		u64::MAX,
	));

	assert_ok!(StableAMM::add_liquidity(
		Origin::signed(USER1),
		1,
		vec![1e18 as Balance, 1e6 as Balance],
		0,
		USER1,
		u64::MAX,
	));
}

fn setup_pools() {
	assert_ok!(Zenlink::create_pair(Origin::root(), TOKEN1_ASSET_ID, TOKEN2_ASSET_ID));
	assert_ok!(Zenlink::add_liquidity(
		Origin::signed(USER1),
		TOKEN1_ASSET_ID,
		TOKEN2_ASSET_ID,
		1e18 as Balance,
		1e18 as Balance,
		0,
		0,
		u64::MAX
	));
}

#[test]
fn swap_exact_token_for_tokens_through_stable_pool_with_amount_slippage_should_failed() {
	new_test_ext().execute_with(|| {
		setup_stable_pools();
		setup_pools();

		let routes = vec![
			Route::Normal(vec![TOKEN2_ASSET_ID, TOKEN1_ASSET_ID]),
			Route::Stable(StablePath::<PoolId, CurrencyId> {
				pool_id: 1,
				base_pool_id: 0,
				mode: FromBase,
				from_currency: Token(TOKEN1_SYMBOL),
				to_currency: Token(TOKEN4_SYMBOL),
			}),
		];

		assert_noop!(
			RouterPallet::swap_exact_token_for_tokens_through_stable_pool(
				Origin::signed(USER2),
				1e16 as Balance,
				u128::MAX,
				routes,
				USER1,
				u64::MAX,
			),
			Error::<Test>::AmountSlippage
		);
	})
}

#[test]
fn swap_exact_token_for_tokens_through_stable_pool_should_work() {
	new_test_ext().execute_with(|| {
		setup_stable_pools();
		setup_pools();

		let routes = vec![
			Route::Normal(vec![TOKEN2_ASSET_ID, TOKEN1_ASSET_ID]),
			Route::Stable(StablePath::<PoolId, CurrencyId> {
				pool_id: 1,
				base_pool_id: 0,
				mode: FromBase,
				from_currency: Token(TOKEN1_SYMBOL),
				to_currency: Token(TOKEN4_SYMBOL),
			}),
		];
		let token1_balance_before = Tokens::accounts(USER1, Token(TOKEN1_SYMBOL)).free;
		let token2_balance_before = Tokens::accounts(USER1, Token(TOKEN2_SYMBOL)).free;
		let token3_balance_before = Tokens::accounts(USER1, Token(TOKEN3_SYMBOL)).free;
		let token4_balance_before = Tokens::accounts(USER2, Token(TOKEN4_SYMBOL)).free;

		assert_ok!(RouterPallet::swap_exact_token_for_tokens_through_stable_pool(
			Origin::signed(USER1),
			1e16 as Balance,
			0,
			routes,
			USER2,
			u64::MAX,
		));

		assert_eq!(
			Tokens::accounts(USER1, Token(TOKEN1_SYMBOL)).free,
			token1_balance_before
		);
		assert_eq!(
			Tokens::accounts(USER1, Token(TOKEN2_SYMBOL)).free,
			token2_balance_before - 1e16 as Balance
		);
		assert_eq!(
			Tokens::accounts(USER1, Token(TOKEN3_SYMBOL)).free,
			token3_balance_before
		);
		assert_eq!(
			Tokens::accounts(USER2, Token(TOKEN4_SYMBOL)).free,
			token4_balance_before + 9854
		);
	})
}
