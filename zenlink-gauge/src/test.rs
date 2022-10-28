use super::*;
use crate::{
	mock::{CurrencyId::*, *},
	Error,
};

use frame_support::{assert_noop, assert_ok};

const VOTE_CURRENCY: CurrencyId = Token(TOKEN1_SYMBOL);
const HOUR: u64 = 3600;
const VOTE_DURATON: u64 = HOUR * 4;
const VOTE_SET_WINDOW: u64 = HOUR;

fn now() -> u64 {
	TimestampPallet::now() / 1000
}

#[test]
fn update_admin_with_root_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(GaugePallet::admin(), None);
		assert_ok!(GaugePallet::update_admin(Origin::root(), ALICE));
		assert_eq!(GaugePallet::admin(), Some(ALICE));
	})
}

#[test]
fn initialize_with_no_admin_should_fail() {
	new_test_ext().execute_with(|| {
		assert_eq!(GaugePallet::admin(), None);
		let start = now();
		assert_noop!(
			GaugePallet::initialize(
				Origin::signed(ALICE),
				VOTE_CURRENCY,
				VOTE_DURATON,
				VOTE_SET_WINDOW,
				start,
			),
			Error::<Test>::OnlyAdmin
		);
	})
}

#[test]
fn initialize_with_old_timestamp_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(GaugePallet::update_admin(Origin::root(), ALICE));

		mine_block();
		let start = now() - 1;

		assert_noop!(
			GaugePallet::initialize(
				Origin::signed(ALICE),
				VOTE_CURRENCY,
				VOTE_DURATON,
				VOTE_SET_WINDOW,
				start,
			),
			Error::<Test>::InvalidTimestamp
		);
	})
}

#[test]
fn initialize_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(GaugePallet::update_admin(Origin::root(), ALICE));

		mine_block();
		let start = now() + 10;

		assert_ok!(GaugePallet::initialize(
			Origin::signed(ALICE),
			VOTE_CURRENCY,
			VOTE_DURATON,
			VOTE_SET_WINDOW,
			start,
		));

		// check storage
		assert_eq!(GaugePallet::next_period_id(), 1);
		assert_eq!(GaugePallet::vote_duration(), VOTE_DURATON);
		assert_eq!(GaugePallet::vote_set_window(), VOTE_SET_WINDOW);
		assert_eq!(GaugePallet::vote_currency(), Some(VOTE_CURRENCY));
		assert_eq!(
			GaugePallet::vote_period(0),
			Some(VotePeriod { start, end: start + VOTE_DURATON })
		)
	})
}

#[test]
fn initialize_repeatedly_should_failed() {
	new_test_ext().execute_with(|| {
		assert_ok!(GaugePallet::update_admin(Origin::root(), ALICE));

		mine_block();
		let start = now() + 10;

		assert_ok!(GaugePallet::initialize(
			Origin::signed(ALICE),
			VOTE_CURRENCY,
			VOTE_DURATON,
			VOTE_SET_WINDOW,
			start,
		));

		assert_ok!(GaugePallet::update_admin(Origin::root(), ALICE));

		mine_block();
		let start = now() + 10;

		assert_noop!(
			GaugePallet::initialize(
				Origin::signed(ALICE),
				VOTE_CURRENCY,
				VOTE_DURATON,
				VOTE_SET_WINDOW,
				start,
			),
			Error::<Test>::Initialized
		);
	})
}

fn initialize_gauge() {
	assert_ok!(GaugePallet::update_admin(Origin::root(), ALICE));

	mine_block();
	let start = now() + 10;

	assert_ok!(GaugePallet::initialize(
		Origin::signed(ALICE),
		VOTE_CURRENCY,
		VOTE_DURATON,
		VOTE_SET_WINDOW,
		start,
	));
}

#[test]
fn set_voteable_pools_with_admin_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();

		let pools = vec![0, 1, 3];

		assert_ok!(GaugePallet::set_voteable_pools(Origin::signed(ALICE), pools.clone()));
		for pid in pools.iter() {
			assert_eq!(GaugePallet::global_pool_state(0, pid).unwrap().votable, true);
		}
	})
}

#[test]
fn set_voteable_pools_with_no_admin_should_failed() {
	new_test_ext().execute_with(|| {
		initialize_gauge();

		let pools = vec![0, 1, 3];

		assert_noop!(
			GaugePallet::set_voteable_pools(Origin::signed(BOB), pools.clone()),
			Error::<Test>::OnlyAdmin
		);

		for pid in pools.iter() {
			assert_eq!(GaugePallet::global_pool_state(0, pid), None);
		}
	})
}

#[test]
fn set_non_voteable_pools_with_admin_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();

		let pools = vec![0, 1, 3];

		assert_ok!(GaugePallet::set_non_voteable_pools(Origin::signed(ALICE), pools.clone()));
		for pid in pools.iter() {
			assert_eq!(GaugePallet::global_pool_state(0, pid).unwrap().votable, false);
		}
	})
}

#[test]
fn set_non_voteable_pools_with_no_admin_should_failed() {
	new_test_ext().execute_with(|| {
		initialize_gauge();

		let pools = vec![0, 1, 3];

		assert_noop!(
			GaugePallet::set_non_voteable_pools(Origin::signed(BOB), pools.clone()),
			Error::<Test>::OnlyAdmin
		);

		for pid in pools.iter() {
			assert_eq!(GaugePallet::global_pool_state(0, pid), None);
		}
	})
}

#[test]
fn update_vote_set_window_with_admin_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		assert_ok!(GaugePallet::update_vote_set_window(
			Origin::signed(ALICE),
			VOTE_SET_WINDOW + 1000
		));

		assert_eq!(GaugePallet::vote_set_window(), VOTE_SET_WINDOW + 1000);
	})
}

#[test]
fn update_vote_set_window_with_no_admin_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		assert_noop!(
			GaugePallet::update_vote_set_window(Origin::signed(BOB), VOTE_SET_WINDOW + 1000),
			Error::<Test>::OnlyAdmin
		);

		assert_eq!(GaugePallet::vote_set_window(), VOTE_SET_WINDOW);
	})
}

#[test]
fn update_vote_duration_with_admin_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		assert_ok!(GaugePallet::update_vote_duration(Origin::signed(ALICE), VOTE_DURATON + 1000));

		assert_eq!(GaugePallet::vote_duration(), VOTE_DURATON + 1000);
	})
}

#[test]
fn update_vote_duration_with_no_admin_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		assert_noop!(
			GaugePallet::update_vote_duration(Origin::signed(BOB), VOTE_SET_WINDOW + 1000),
			Error::<Test>::OnlyAdmin
		);

		assert_eq!(GaugePallet::vote_duration(), VOTE_DURATON);
	})
}

#[test]
fn update_vote_period_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		let period0 = GaugePallet::vote_period(0).unwrap();
		assert_ok!(GaugePallet::update_vote_period(Origin::signed(BOB)));

		// in period0, updateVotePeriod do nothing
		let mut next_block_timestamp = period0.start + HOUR;
		set_block_timestamp(next_block_timestamp);
		assert_ok!(GaugePallet::update_vote_period(Origin::signed(BOB)));

		// after period0, update vote period success
		next_block_timestamp = period0.end + VOTE_SET_WINDOW / 2;
		set_block_timestamp(next_block_timestamp);
		assert_ok!(GaugePallet::update_vote_period(Origin::signed(BOB)));

		let period1 = GaugePallet::vote_period(1).unwrap();
		let next_period_start = period0.end + VOTE_SET_WINDOW;
		let next_period_end = next_period_start + VOTE_DURATON;
		assert_eq!(period1.start, next_period_start);
		assert_eq!(period1.end, next_period_end);

		// after period1.end + voteSetWindow, period2.start = block.Timestamp
		next_block_timestamp = period1.end + VOTE_SET_WINDOW + 10;
		set_block_timestamp(next_block_timestamp);
		assert_ok!(GaugePallet::update_vote_period(Origin::signed(BOB)));
		let period2 = GaugePallet::vote_period(2).unwrap();
		let current_timestmap = now();
		assert_eq!(period2.start, current_timestmap);
		assert_eq!(period2.end, current_timestmap + VOTE_DURATON);
	})
}

#[test]
fn update_vote_period_params_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		let period0 = GaugePallet::vote_period(0).unwrap();
		assert_ok!(GaugePallet::update_vote_duration(Origin::signed(ALICE), VOTE_DURATON - HOUR));
		assert_ok!(GaugePallet::update_vote_set_window(
			Origin::signed(ALICE),
			VOTE_SET_WINDOW + HOUR
		));

		// The period 0 is not expired, so nothing changed.
		assert_ok!(GaugePallet::update_vote_period(Origin::signed(BOB)));

		let period0_after_update = GaugePallet::vote_period(0).unwrap();
		assert_eq!(period0, period0_after_update);

		let next_block_timestamp = period0.start + VOTE_DURATON + HOUR + 10;
		set_block_timestamp(next_block_timestamp);

		assert_ok!(GaugePallet::update_vote_period(Origin::signed(BOB)));
		let period1 = GaugePallet::vote_period(1).unwrap();
		assert_eq!(period1.start, period0.end + VOTE_SET_WINDOW + HOUR);
		assert_eq!(period1.end, period1.start + VOTE_DURATON - HOUR);
	})
}

#[test]
fn update_pool_votable_at_any_time_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		// in period0
		assert_ok!(GaugePallet::set_voteable_pools(Origin::signed(ALICE), vec![1]));
		assert_eq!(GaugePallet::global_pool_state(0, 1).unwrap().reset_votable, true);

		//after period0
		let period0 = GaugePallet::vote_period(0).unwrap();
		let mut next_timestamp = period0.end + 10;
		set_block_timestamp(next_timestamp);
		assert_ok!(GaugePallet::set_voteable_pools(Origin::signed(ALICE), vec![1]));
		let mut pool_state = GaugePallet::global_pool_state(1, 1).unwrap();
		assert_eq!(pool_state.reset_votable, true);
		assert_eq!(pool_state.votable, true);

		assert_ok!(GaugePallet::set_non_voteable_pools(Origin::signed(ALICE), vec![1]));
		pool_state = GaugePallet::global_pool_state(1, 1).unwrap();
		assert_eq!(pool_state.reset_votable, true);
		assert_eq!(pool_state.votable, false);

		// after period0.End + voteSetWindow
		next_timestamp = period0.end + VOTE_SET_WINDOW + 10;
		set_block_timestamp(next_timestamp);
		pool_state = GaugePallet::global_pool_state(1, 1).unwrap();
		assert_eq!(pool_state.reset_votable, true);
		assert_eq!(pool_state.votable, false);

		assert_eq!(GaugePallet::next_period_id(), 1);
	})
}

#[test]
fn admin_update_vote_not_overwrite_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();

		let period0 = GaugePallet::vote_period(0).unwrap();
		let next_timestamp = period0.end + 10;
		set_block_timestamp(next_timestamp);

		assert_ok!(GaugePallet::set_voteable_pools(Origin::signed(ALICE), vec![1]));
		let mut pool_state = GaugePallet::global_pool_state(1, 1).unwrap();
		assert_eq!(pool_state.reset_votable, true);
		assert_eq!(pool_state.votable, true);

		assert_ok!(GaugePallet::update_vote_period(Origin::signed(BOB)));
		assert_ok!(GaugePallet::update_pool_histroy(Origin::signed(BOB), 1, 1));

		pool_state = GaugePallet::global_pool_state(1, 1).unwrap();
		assert_eq!(pool_state.reset_votable, true);
		assert_eq!(pool_state.votable, true);
	})
}

fn calculate_score(current_timestamp: Timestamp, amount: Balance, period: &VotePeriod) -> Balance {
	amount * (period.end - current_timestamp) as Balance / (period.end - period.start) as Balance
}

#[test]
fn vote_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		// vote before period0 start
		assert_noop!(
			GaugePallet::vote(Origin::signed(BOB), 1, 100 * TOKEN1_UNIT),
			Error::<Test>::NonVotablePool
		);
		assert_ok!(GaugePallet::set_voteable_pools(Origin::signed(ALICE), vec![1]));
		assert_ok!(GaugePallet::vote(Origin::signed(BOB), 1, 10 * TOKEN1_UNIT));

		assert_eq!(GaugePallet::account_vote_amount(BOB, 1), Some(10 * TOKEN1_UNIT));

		let mut pool_state = GaugePallet::global_pool_state(0, 1).unwrap();
		assert_eq!(pool_state.total_amount, 10 * TOKEN1_UNIT);
		assert_eq!(pool_state.score, 10 * TOKEN1_UNIT);

		// after period0 start
		let period0 = GaugePallet::vote_period(0).unwrap();
		let mut next_block_timestamp = period0.start + HOUR + 2;
		set_block_timestamp(next_block_timestamp);

		assert_ok!(GaugePallet::vote(Origin::signed(BOB), 1, 10 * TOKEN1_UNIT));

		assert_eq!(GaugePallet::account_vote_amount(BOB, 1), Some(20 * TOKEN1_UNIT));

		pool_state = GaugePallet::global_pool_state(0, 1).unwrap();
		let period0_score =
			10 * TOKEN1_UNIT + calculate_score(next_block_timestamp, 10 * TOKEN1_UNIT, &period0);
		assert_eq!(pool_state.score, period0_score);
		assert_eq!(pool_state.total_amount, 20 * TOKEN1_UNIT);

		next_block_timestamp = period0.end + 10;
		set_block_timestamp(next_block_timestamp);
		// vote after period0 end, it vote to period1
		assert_ok!(GaugePallet::vote(Origin::signed(BOB), 1, 10 * TOKEN1_UNIT));

		// the period has been update
		assert_eq!(GaugePallet::next_period_id(), 2);
		let period1 = GaugePallet::vote_period(1).unwrap();
		assert_eq!(
			period1,
			VotePeriod {
				start: period0.end + VOTE_SET_WINDOW,
				end: period0.end + VOTE_SET_WINDOW + VOTE_DURATON
			}
		);

		pool_state = GaugePallet::global_pool_state(1, 1).unwrap();
		assert_eq!(pool_state.score, 30 * TOKEN1_UNIT);
		assert_eq!(pool_state.total_amount, 30 * TOKEN1_UNIT);

		// pool state in period0 should no change after period0 end.
		pool_state = GaugePallet::global_pool_state(0, 1).unwrap();
		assert_eq!(pool_state.score, period0_score);
		assert_eq!(pool_state.total_amount, 20 * TOKEN1_UNIT);

		assert_eq!(GaugePallet::account_vote_amount(BOB, 1), Some(30 * TOKEN1_UNIT));
		assert_eq!(get_user_balance(VOTE_CURRENCY, &BOB), 0);
	})
}

#[test]
fn update_pool_histroy_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		assert_ok!(GaugePallet::set_voteable_pools(Origin::signed(ALICE), vec![1]));
		assert_ok!(GaugePallet::vote(Origin::signed(BOB), 1, 10 * TOKEN1_UNIT));

		let mut pool_state = GaugePallet::global_pool_state(0, 1).unwrap();
		assert_eq!(pool_state.total_amount, 10 * TOKEN1_UNIT);
		assert_eq!(pool_state.score, 10 * TOKEN1_UNIT);

		for i in 1..=5 {
			let period = GaugePallet::vote_period(i - 1).unwrap();
			let next_period_start = period.end + VOTE_SET_WINDOW;
			let next_period_end = next_period_start + VOTE_DURATON;
			let next_block_timestamp = next_period_end + 10;
			set_block_timestamp(next_block_timestamp);
			assert_ok!(GaugePallet::update_vote_period(Origin::signed(BOB)));
		}

		assert_ok!(GaugePallet::update_pool_histroy(Origin::signed(BOB), 1, 2));

		for i in 1..=2 {
			pool_state = GaugePallet::global_pool_state(i, 1).unwrap();
			assert_eq!(pool_state.total_amount, 10 * TOKEN1_UNIT);
			assert_eq!(pool_state.score, 10 * TOKEN1_UNIT);
			assert_eq!(pool_state.inherit, true);
			assert_eq!(pool_state.votable, true);
		}

		for i in 3..=5 {
			assert_eq!(GaugePallet::global_pool_state(i, 1), None);
		}

		assert_ok!(GaugePallet::update_pool_histroy(Origin::signed(BOB), 1, 4));

		for i in 3..=4 {
			pool_state = GaugePallet::global_pool_state(i, 1).unwrap();
			assert_eq!(pool_state.total_amount, 10 * TOKEN1_UNIT);
			assert_eq!(pool_state.score, 10 * TOKEN1_UNIT);
			assert_eq!(pool_state.inherit, true);
			assert_eq!(pool_state.votable, true);
		}

		assert_eq!(GaugePallet::global_pool_state(5, 1), None);
	})
}

#[test]
fn cancel_vote_should_work() {
	new_test_ext().execute_with(|| {
		initialize_gauge();
		assert_ok!(GaugePallet::set_voteable_pools(Origin::signed(ALICE), vec![1]));
		assert_ok!(GaugePallet::vote(Origin::signed(BOB), 1, 10 * TOKEN1_UNIT));
		assert_ok!(GaugePallet::cancel_vote(Origin::signed(BOB), 1, 10 * TOKEN1_UNIT));
		let mut pool_state = GaugePallet::global_pool_state(0, 1).unwrap();

		assert_eq!(pool_state.total_amount, 0);
		assert_eq!(pool_state.score, 0);
		assert_eq!(get_user_balance(VOTE_CURRENCY, &BOB), 30 * TOKEN1_UNIT);

		// cancel vote in period0
		let period0 = GaugePallet::vote_period(0).unwrap();
		let mut next_block_timestamp = period0.start + HOUR * 2;
		set_block_timestamp(next_block_timestamp);
		assert_ok!(GaugePallet::vote(Origin::signed(BOB), 1, 10 * TOKEN1_UNIT));
		let added_score = calculate_score(next_block_timestamp, 10 * TOKEN1_UNIT, &period0);

		next_block_timestamp = period0.start + HOUR * 2;
		set_block_timestamp(next_block_timestamp);
		assert_ok!(GaugePallet::cancel_vote(Origin::signed(BOB), 1, 5 * TOKEN1_UNIT));
		let removed_score = calculate_score(next_block_timestamp, 5 * TOKEN1_UNIT, &period0);

		pool_state = GaugePallet::global_pool_state(0, 1).unwrap();

		assert_eq!(pool_state.total_amount, 5 * TOKEN1_UNIT);
		assert_eq!(pool_state.score, added_score - removed_score);
		assert_eq!(get_user_balance(VOTE_CURRENCY, &BOB), 25 * TOKEN1_UNIT);

		// cancel vote after period0
		next_block_timestamp = period0.end + 10;
		set_block_timestamp(next_block_timestamp);
		assert_ok!(GaugePallet::cancel_vote(Origin::signed(BOB), 1, 5 * TOKEN1_UNIT));
		// pool1 change nothing in period0
		pool_state = GaugePallet::global_pool_state(0, 1).unwrap();
		assert_eq!(pool_state.total_amount, 5 * TOKEN1_UNIT);
		assert_eq!(pool_state.score, added_score - removed_score);

		assert_eq!(GaugePallet::next_period_id(), 2);

		assert_eq!(get_user_balance(VOTE_CURRENCY, &BOB), 30 * TOKEN1_UNIT);
		pool_state = GaugePallet::global_pool_state(1, 1).unwrap();
		assert_eq!(pool_state.total_amount, 0);
		assert_eq!(pool_state.score, 0);
	})
}
