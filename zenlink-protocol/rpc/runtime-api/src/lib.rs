#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_std::vec::Vec;
use zenlink_protocol::{AssetId, PairInfo, TokenBalance};

sp_api::decl_runtime_apis! {
	 pub trait ZenlinkProtocolApi<AccountId>
	 where
		AccountId: Codec,
		TokenBalance: Codec
	 {
		fn get_assets() -> Vec<AssetId>;

		fn get_balance(asset_id: AssetId, owner: AccountId) -> TokenBalance;

		fn get_all_pairs() -> Vec<PairInfo<AccountId, TokenBalance>>;

		fn get_owner_pairs(owner: AccountId) -> Vec<PairInfo<AccountId, TokenBalance>>;

		//buy amount token price
		fn get_amount_in_price(path: Vec<AssetId>) -> TokenBalance;

		//sell amount token price
		fn get_amount_out_price(path: Vec<AssetId>) -> TokenBalance;
	 }
}
