#![allow(clippy::type_complexity)]

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

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
            .filter_map(|(token_0, token_1)|{ Self::get_pair_from_asset_id(token_0, token_1) })
            .map(|pair|
                PairInfo {
                    token_0: pair.token_0,
                    token_1: pair.token_1,
                    account: pair.account.clone(),
                    total_liquidity: pair.total_liquidity,
                    holding_liquidity: TokenBalance::default(),
                    reserve_0: Self::asset_balance_of(&pair.token_0, &pair.account),
                    reserve_1: Self::asset_balance_of(&pair.token_1, &pair.account),
                }
            ).collect::<Vec<_>>()
    }

    pub fn get_owner_pairs(owner: &T::AccountId) -> Vec<PairInfo<T::AccountId, TokenBalance>> {
        <Pairs>::get()
            .iter()
            .filter_map( |(token_0, token_1)|{ Self::get_pair_from_asset_id(token_0, token_1) })
            .filter_map(|pair| {
                    let hold =  <LiquidityPool<T>>::get((pair.account.clone(), owner));
                    if hold > 0{
                        Some(PairInfo {
                            token_0: pair.token_0,
                            token_1: pair.token_1,
                            account: pair.account.clone(),
                            total_liquidity: pair.total_liquidity,
                            holding_liquidity: hold,
                            reserve_0: Self::asset_balance_of(&pair.token_0, &pair.account),
                            reserve_1: Self::asset_balance_of(&pair.token_1, &pair.account),
                        })
                    }else{
                        None
                    }
                }
            ).collect::<Vec<_>>()
    }

    //buy amount token price
    pub fn get_in_price(path: Vec<AssetId>) -> TokenBalance {
        let amount_in_unit = 100_000_000_u32;
        let  amount_in_unit = TokenBalance::from(amount_in_unit);
        Self::get_amount_out_by_path(amount_in_unit, &path) .map_or(TokenBalance::default(), |amounts| {
                *amounts.last().unwrap_or(&TokenBalance::default())
            })
    }

    //sell amount token price
    pub fn get_out_price(path: Vec<AssetId>) -> TokenBalance {
        let amount_out_unit = 100_000_000_u32;
        let  amount_out_unit = TokenBalance::from(amount_out_unit);
        Self::get_amount_in_by_path(amount_out_unit, &path) .map_or(TokenBalance::default(), |amounts| {
            *amounts.first().unwrap_or(&TokenBalance::default())
        })
    }
}
