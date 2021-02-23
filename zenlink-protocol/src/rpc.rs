// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

#![allow(clippy::type_complexity)]

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct PairInfo<AccountId, TokenBalance> {
	pub token_0: AssetId,
	pub token_1: AssetId,

	pub account: AccountId,
	pub total_liquidity: TokenBalance,
	pub holding_liquidity: TokenBalance,
	pub reserve_0: TokenBalance,
	pub reserve_1: TokenBalance,
}

impl<T: Config> Module<T> {
	pub fn get_all_pairs() -> Vec<PairInfo<T::AccountId, TokenBalance>> {
		<Pairs>::get()
			.iter()
			.filter_map(|(token_0, token_1)| Self::get_pair_from_asset_id(token_0, token_1))
			.map(|pair| PairInfo {
				token_0: pair.token_0,
				token_1: pair.token_1,
				account: pair.account.clone(),
				total_liquidity: pair.total_liquidity,
				holding_liquidity: TokenBalance::default(),
				reserve_0: Self::asset_balance_of(&pair.token_0, &pair.account),
				reserve_1: Self::asset_balance_of(&pair.token_1, &pair.account),
			})
			.collect::<Vec<_>>()
	}

	pub fn get_owner_pairs(owner: &T::AccountId) -> Vec<PairInfo<T::AccountId, TokenBalance>> {
		<Pairs>::get()
			.iter()
			.filter_map(|(token_0, token_1)| Self::get_pair_from_asset_id(token_0, token_1))
			.filter_map(|pair| {
				let hold = <LiquidityPool<T>>::get((pair.account.clone(), owner));
				if hold > 0 {
					Some(PairInfo {
						token_0: pair.token_0,
						token_1: pair.token_1,
						account: pair.account.clone(),
						total_liquidity: pair.total_liquidity,
						holding_liquidity: hold,
						reserve_0: Self::asset_balance_of(&pair.token_0, &pair.account),
						reserve_1: Self::asset_balance_of(&pair.token_1, &pair.account),
					})
				} else {
					None
				}
			})
			.collect::<Vec<_>>()
	}

	//buy amount token price
	pub fn get_in_price(supply: TokenBalance, path: Vec<AssetId>) -> TokenBalance {
		Self::get_amount_out_by_path(supply, &path).map_or(TokenBalance::default(), |amounts| {
			*amounts.last().unwrap_or(&TokenBalance::default())
		})
	}

	//sell amount token price
	pub fn get_out_price(supply: TokenBalance, path: Vec<AssetId>) -> TokenBalance {
		Self::get_amount_in_by_path(supply, &path).map_or(TokenBalance::default(), |amounts| {
			*amounts.first().unwrap_or(&TokenBalance::default())
		})
	}

	pub fn get_estimate_lptoken(
		token_0: AssetId,
		token_1: AssetId,
		amount_0_desired: TokenBalance,
		amount_1_desired: TokenBalance,
		amount_0_min: TokenBalance,
		amount_1_min: TokenBalance,
	) -> TokenBalance {
		Self::get_pair_from_asset_id(&token_0, &token_1).map_or(TokenBalance::default(), |pair| {
			let reserve_0 = Self::asset_balance_of(&token_0, &pair.account);
			let reserve_1 = Self::asset_balance_of(&token_1, &pair.account);

			Self::calculate_added_amount(
				amount_0_desired,
				amount_1_desired,
				amount_0_min,
				amount_1_min,
				reserve_0,
				reserve_1,
			)
			.map_or(TokenBalance::default(), |(amount_0, amount_1)| {
				Self::mint_liquidity(
					amount_0,
					amount_1,
					reserve_0,
					reserve_1,
					pair.total_liquidity,
				)
			})
		})
	}
}
