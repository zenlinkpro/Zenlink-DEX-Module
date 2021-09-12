// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use super::*;
use sp_runtime::FixedU128;

pub type AssetBalance = u128;
pub type Rate = FixedU128;

/// Native currency
pub const NATIVE: u8 = 0;
/// Swap module asset
pub const LIQUIDITY: u8 = 1;
/// Other asset type on this chain
pub const LOCAL: u8 = 2;
/// Reserved for future
pub const RESERVED: u8 = 3;

/// AssetId use to locate assets in framed base chain.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Default))]
pub struct AssetId {
	pub chain_id: u32,
	pub asset_type: u8,
	pub asset_index: u64,
}

impl AssetId {
	pub fn is_support(&self) -> bool {
		matches!(self.asset_type, NATIVE | LIQUIDITY | LOCAL | RESERVED)
	}

	pub fn is_native(&self, self_chain_id: u32) -> bool {
		self.chain_id == self_chain_id && self.asset_type == NATIVE && self.asset_index == 0
	}

	pub fn is_foreign(&self, self_chain_id: u32) -> bool {
		self.chain_id != self_chain_id
	}
}

/// Status for TradingPair
#[derive(Clone, Copy, Encode, Decode, RuntimeDebug, PartialEq, Eq, MaxEncodedLen)]
pub enum PairStatus<Balance, BlockNumber, Account> {
	/// Pair is Trading,
	/// can add/remove liquidity and swap.
	Trading(PairMetadata<Balance, Account>),
	/// pair is Bootstrap,
	/// can add liquidity.
	Bootstrap(BootstrapParameter<Balance, BlockNumber, Account>),
	/// nothing in pair
	Disable,
}

impl<Balance, BlockNumber, Account> Default for PairStatus<Balance, BlockNumber, Account> {
	fn default() -> Self {
		Self::Disable
	}
}

/// Parameters of pair in Bootstrap status
#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, MaxEncodedLen)]
pub struct BootstrapParameter<Balance, BlockNumber, Account> {
	/// limit contribution per time.
	pub min_contribution: (Balance, Balance),
	/// target supply that trading pair could to normal.
	pub target_supply: (Balance, Balance),
	/// accumulated supply for this Bootstrap pair.
	pub accumulated_supply: (Balance, Balance),
	/// bootstrap pair end block number.
	pub end_block_number: BlockNumber,
	/// bootstrap pair account.
	pub pair_account: Account,
}

#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, MaxEncodedLen)]
pub struct PairMetadata<Balance, Account> {
	pub pair_account: Account,
	pub total_supply: Balance,
}
