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
pub struct PairInfo<AccountId, AssetBalance> {
	pub asset_0: AssetId,
	pub asset_1: AssetId,

	pub account: AccountId,
	pub total_liquidity: AssetBalance,
	pub holding_liquidity: AssetBalance,
	pub reserve_0: AssetBalance,
	pub reserve_1: AssetBalance,
	pub lp_asset_id: AssetId,
}

impl<T: Config> Pallet<T> {
	pub fn get_pair_by_asset_id(asset_0: AssetId, asset_1: AssetId) -> Option<PairInfo<T::AccountId, AssetBalance>> {
		let pair_account = Self::pair_account_id(asset_0, asset_1);
		let lp_asset_id = Self::lp_asset_id(&asset_0, &asset_1);

		Some(PairInfo {
			asset_0,
			asset_1,
			account: pair_account.clone(),
			total_liquidity: T::MultiAssetsHandler::total_supply(lp_asset_id),
			holding_liquidity: Zero::zero(),
			reserve_0: T::MultiAssetsHandler::balance_of(asset_0, &pair_account),
			reserve_1: T::MultiAssetsHandler::balance_of(asset_1, &pair_account),
			lp_asset_id,
		})
	}

	pub fn get_sovereigns_info(asset_id: &AssetId) -> Vec<(u32, T::AccountId, AssetBalance)> {
		T::TargetChains::get()
			.iter()
			.filter_map(|(location, _)| match location {
				MultiLocation::X2(Junction::Parent, Junction::Parachain(id)) => {
					if let Ok(sovereign) = T::Conversion::convert_ref(location) {
						Some((*id, sovereign))
					} else {
						None
					}
				}
				_ => None,
			})
			.map(|(para_id, account)| {
				let balance = T::MultiAssetsHandler::balance_of(*asset_id, &account);

				(para_id, account, balance)
			})
			.collect::<Vec<_>>()
	}
}
