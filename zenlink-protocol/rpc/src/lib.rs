// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! RPC interface for the zenlink dex module.
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

use zenlink_protocol::{AssetId, PairInfo, TokenBalance};
use zenlink_protocol_runtime_api::ZenlinkProtocolApi as ZenlinkProtocolRuntimeApi;

#[rpc]
pub trait ZenlinkProtocolApi<BlockHash, AccountId> {
    #[rpc(name = "zenlinkProtocol_getAllAssets")]
    fn get_assets(&self, at: Option<BlockHash>) -> Result<Vec<AssetId>>;

    #[rpc(name = "zenlinkProtocol_getBalance")]
    fn get_balance(
        &self,
        asset_id: AssetId,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Result<NumberOrHex>;

    #[rpc(name = "zenlinkProtocol_getAllPairs")]
    fn get_all_pairs(&self, at: Option<BlockHash>)
        -> Result<Vec<PairInfo<AccountId, NumberOrHex>>>;

    #[rpc(name = "zenlinkProtocol_getOwnerPairs")]
    fn get_owner_pairs(
        &self,
        owner: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<PairInfo<AccountId, NumberOrHex>>>;

    #[rpc(name = "zenlinkProtocol_getPairByAssetId")]
    fn get_pair_by_asset_id(
        &self,
        token_0: AssetId,
        token_1: AssetId,
        at: Option<BlockHash>,
    ) -> Result<Option<PairInfo<AccountId, NumberOrHex>>>;

    #[rpc(name = "zenlinkProtocol_getAmountInPrice")]
    fn get_amount_in_price(
        &self,
        supply: TokenBalance,
        path: Vec<AssetId>,
        at: Option<BlockHash>,
    ) -> Result<NumberOrHex>;

    #[rpc(name = "zenlinkProtocol_getAmountOutPrice")]
    fn get_amount_out_price(
        &self,
        supply: TokenBalance,
        path: Vec<AssetId>,
        at: Option<BlockHash>,
    ) -> Result<NumberOrHex>;

    #[rpc(name = "zenlinkProtocol_getEstimateLptoken")]
    fn get_estimate_lptoken(
        &self,
        token_0: AssetId,
        token_1: AssetId,
        amount_0_desired: TokenBalance,
        amount_1_desired: TokenBalance,
        amount_0_min: TokenBalance,
        amount_1_min: TokenBalance,
        at: Option<BlockHash>,
    ) -> Result<NumberOrHex>;
}

const RUNTIME_ERROR: i64 = 1;

pub struct ZenlinkProtocol<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> ZenlinkProtocol<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

impl<C, Block, AccountId> ZenlinkProtocolApi<<Block as BlockT>::Hash, AccountId>
    for ZenlinkProtocol<C, Block>
where
    Block: BlockT,
    AccountId: Codec,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: ZenlinkProtocolRuntimeApi<Block, AccountId>,
{
    fn get_assets(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<AssetId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        Ok(api.get_assets(&at).map_err(runtime_error_into_rpc_err)?)
    }

    fn get_balance(
        &self,
        asset_id: AssetId,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NumberOrHex> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        Ok(api
            .get_balance(&at, asset_id, account)
            .map(|token_balance| token_balance.into())
            .map_err(runtime_error_into_rpc_err)?)
    }

    fn get_all_pairs(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<PairInfo<AccountId, NumberOrHex>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        Ok(api
            .get_all_pairs(&at)
            .map(|pairs| {
                pairs
                    .into_iter()
                    .map(|pair| PairInfo {
                        token_0: pair.token_0,
                        token_1: pair.token_1,
                        account: pair.account,
                        total_liquidity: pair.total_liquidity.into(),
                        holding_liquidity: pair.holding_liquidity.into(),
                        reserve_0: pair.reserve_0.into(),
                        reserve_1: pair.reserve_1.into(),
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(runtime_error_into_rpc_err)?)
    }

    fn get_owner_pairs(
        &self,
        owner: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<PairInfo<AccountId, NumberOrHex>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        Ok(api
            .get_owner_pairs(&at, owner)
            .map(|pairs| {
                pairs
                    .into_iter()
                    .map(|pair| PairInfo {
                        token_0: pair.token_0,
                        token_1: pair.token_1,
                        account: pair.account,
                        total_liquidity: pair.total_liquidity.into(),
                        holding_liquidity: pair.holding_liquidity.into(),
                        reserve_0: pair.reserve_0.into(),
                        reserve_1: pair.reserve_1.into(),
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(runtime_error_into_rpc_err)?)
    }

    fn get_pair_by_asset_id(
        &self,
        token_0: AssetId,
        token_1: AssetId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<PairInfo<AccountId, NumberOrHex>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        Ok(api
            .get_all_pairs(&at)
            .map(|pairs| {
                pairs
                    .into_iter()
                    .find(|pair| {
                        (pair.token_0 == token_0 && pair.token_1 == token_1)
                            || (pair.token_0 == token_1 && pair.token_1 == token_0)
                    })
                    .map(|pair| PairInfo {
                        token_0: pair.token_0,
                        token_1: pair.token_1,
                        account: pair.account,
                        total_liquidity: pair.total_liquidity.into(),
                        holding_liquidity: pair.holding_liquidity.into(),
                        reserve_0: pair.reserve_0.into(),
                        reserve_1: pair.reserve_1.into(),
                    })
            })
            .map_err(runtime_error_into_rpc_err)?)
    }

    //buy amount token price
    fn get_amount_in_price(
        &self,
        supply: TokenBalance,
        path: Vec<AssetId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NumberOrHex> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        Ok(api
            .get_amount_in_price(&at, supply, path)
            .map(|price| price.into())
            .map_err(runtime_error_into_rpc_err)?)
    }

    //sell amount token price
    fn get_amount_out_price(
        &self,
        supply: TokenBalance,
        path: Vec<AssetId>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NumberOrHex> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        Ok(api
            .get_amount_out_price(&at, supply, path)
            .map(|price| price.into())
            .map_err(runtime_error_into_rpc_err)?)
    }

    fn get_estimate_lptoken(
        &self,
        token_0: AssetId,
        token_1: AssetId,
        amount_0_desired: TokenBalance,
        amount_1_desired: TokenBalance,
        amount_0_min: TokenBalance,
        amount_1_min: TokenBalance,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NumberOrHex> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        Ok(api
            .get_estimate_lptoken(
                &at,
                token_0,
                token_1,
                amount_0_desired,
                amount_1_desired,
                amount_0_min,
                amount_1_min,
            )
            .map(|price| price.into())
            .map_err(runtime_error_into_rpc_err)?)
    }
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(RUNTIME_ERROR),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
