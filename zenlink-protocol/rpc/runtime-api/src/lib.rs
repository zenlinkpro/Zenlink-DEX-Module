// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_std::vec::Vec;
use zenlink_protocol::{AssetBalance, AssetId, PairInfo};
use sp_core::{H256};

sp_api::decl_runtime_apis! {
	 pub trait ZenlinkProtocolApi<AccountId>
	 where
		AccountId: Codec,
		AssetBalance: Codec
	 {

		fn get_balance(asset_id: AssetId, owner: AccountId) -> AssetBalance;

		fn get_sovereigns_info(asset_id: AssetId) -> Vec<(u32, AccountId, AssetBalance)>;

		fn get_pair_by_asset_id(
			asset_0: AssetId,
			asset_1: AssetId
		) -> Option<PairInfo<AccountId, AssetBalance>>;

		//buy amount asset price
		fn get_amount_in_price(supply: AssetBalance, path: Vec<AssetId>) -> AssetBalance;

		//sell amount asset price
		fn get_amount_out_price(supply: AssetBalance, path: Vec<AssetId>) -> AssetBalance;

		fn get_estimate_lptoken(
			asset_0: AssetId,
			asset_1: AssetId,
			amount_0_desired: AssetBalance,
			amount_1_desired: AssetBalance,
			amount_0_min: AssetBalance,
			amount_1_min: AssetBalance,
		) -> AssetBalance;

		fn hashes_of_maker(
			maker: AccountId,
		 	page:  u64,
		 	limit: u64,
	 	)->Vec<H256>;

		fn hashes_of_maker_invert(
			maker: AccountId,
		 	page:  u64,
		 	limit: u64,
	 	)->Vec<H256>;

		fn hashes_of_from_token(
		 	maker: AssetId,
		 	page:  u64,
		 	limit: u64,
	 	)->Vec<H256>;

		fn hashes_of_to_token(
		 	maker: AssetId,
		 	page:  u64,
		 	limit: u64,
	 	) ->Vec<H256>;

		fn all_hashes(
		 	page : u64,
		 	limit: u64,
	 	)-> Vec<H256>;
	 }
}
