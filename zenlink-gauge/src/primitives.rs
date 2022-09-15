use super::*;
use sp_arithmetic::{helpers_128bit::*, Rounding};

pub type Balance = u128;
pub type PeriodId = u32;
pub type Duration = u64;
pub type Timestamp = u64;

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct VotePeriod {
	// The start timestmap of a vote period.
	pub start: u64,
	// The end timestmap of a vote period.
	pub end: u64,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct PoolState {
	// Flag marking whether the pool inherit the last period token.
	pub inherit: bool,
	// Flag marking whether the pool votable has been reset by admin.
	pub reset_votable: bool,
	// Flag marking whether the pool is votable in this period.
	pub votable: bool,
	// The score this pool get in this period.
	pub score: Balance,
	// The Amount of token this pool get in this period.
	pub total_amount: Balance,
}

pub fn balance_mul_div(x: Balance, y: Balance, z: Balance) -> Option<Balance> {
	multiply_by_rational_with_rounding(x, y, z, Rounding::Down)
}
