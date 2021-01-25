use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{DispatchResult, RuntimeDebug};

use frame_support::traits::Vec;

/// The balance of zenlink asset
pub type TokenBalance = u128;

/// The pair id of the zenlink dex.
pub type PairId = u32;

/// The id of Zenlink asset
/// NativeCurrency is this parachain native currency.
/// Other parachain's currency is represented by `ParaCurrency(u32)`, `u32` cast to the ParaId.
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetId {
    NativeCurrency,
    ParaCurrency(u32),
}

impl AssetId {
    pub fn is_para_currency(&self) -> bool {
        matches!(self, AssetId::ParaCurrency(_))
    }
}

impl From<u32> for AssetId {
    fn from(id: u32) -> Self {
        AssetId::ParaCurrency(id)
    }
}

impl From<u128> for AssetId {
    fn from(id: u128) -> Self {
        AssetId::ParaCurrency(id as u32)
    }
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug)]
pub enum CrossChainOperation
{
    AddLiquidity { origin_chain: u32, target_chain: u32, token_0: u32, token_1: u32, amount_1: TokenBalance },

    SwapExactTokensForTokens { origin_chain: u32, target_chain: u32, amount_out_min: TokenBalance, path: Vec<u32> },

    SwapTokensForExactTokens { origin_chain: u32, target_chain: u32, amount_out: TokenBalance, path: Vec<u32> },
}

pub trait MultiAsset<AccountId, TokenBalance> {
    fn total_supply(asset_id: AssetId) -> TokenBalance;

    fn balance_of(asset_id: AssetId, who: &AccountId) -> TokenBalance;

    fn transfer(
        asset_id: AssetId,
        from: &AccountId,
        to: &AccountId,
        amount: TokenBalance,
    ) -> DispatchResult;

    fn withdraw(asset_id: AssetId, who: &AccountId, amount: TokenBalance) -> DispatchResult;

    fn deposit(asset_id: AssetId, who: &AccountId, amount: TokenBalance) -> DispatchResult;
}

pub trait OperateAsset<AccountId, TokenBalance>
{
    fn add_liquidity(
        who: &AccountId,
        token_0: &AssetId,
        token_1: &AssetId,
        amount_0_desired: TokenBalance,
        amount_1_desired: TokenBalance,
    ) -> DispatchResult;

    fn swap_in(
        who: &AccountId,
        amount_in: TokenBalance,
        amount_out_min: TokenBalance,
        path: &[AssetId],
    ) -> DispatchResult;

    fn swap_out(
        who: &AccountId,
        amount_out: TokenBalance,
        amount_in_max: TokenBalance,
        path: &[AssetId],
    ) -> DispatchResult;

    fn restore_parachain_asset(
        who: &AccountId,
        amount_out: TokenBalance,
        asset_id: &AssetId,
    ) -> DispatchResult;
}

