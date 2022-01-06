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
use sp_core::H256;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

use zenlink_protocol::{AssetBalance, AssetId, PairInfo};
use zenlink_protocol_runtime_api::ZenlinkProtocolApi as ZenlinkProtocolRuntimeApi;

#[rpc]
pub trait ZenlinkProtocolApi<BlockHash, AccountId> {
	#[rpc(name = "zenlinkProtocol_getBalance")]
	fn get_balance(&self, asset_id: AssetId, account: AccountId, at: Option<BlockHash>) -> Result<NumberOrHex>;

	#[rpc(name = "zenlinkProtocol_getSovereignsInfo")]
	fn get_sovereigns_info(
		&self,
		asset_id: AssetId,
		at: Option<BlockHash>,
	) -> Result<Vec<(u32, AccountId, NumberOrHex)>>;

	#[rpc(name = "zenlinkProtocol_getPairByAssetId")]
	fn get_pair_by_asset_id(
		&self,
		asset_0: AssetId,
		asset_1: AssetId,
		at: Option<BlockHash>,
	) -> Result<Option<PairInfo<AccountId, NumberOrHex>>>;

	#[rpc(name = "zenlinkProtocol_getAmountInPrice")]
	fn get_amount_in_price(
		&self,
		supply: AssetBalance,
		path: Vec<AssetId>,
		at: Option<BlockHash>,
	) -> Result<NumberOrHex>;

	#[rpc(name = "zenlinkProtocol_getAmountOutPrice")]
	fn get_amount_out_price(
		&self,
		supply: AssetBalance,
		path: Vec<AssetId>,
		at: Option<BlockHash>,
	) -> Result<NumberOrHex>;

	#[rpc(name = "zenlinkProtocol_getEstimateLptoken")]
	fn get_estimate_lptoken(
		&self,
		asset_0: AssetId,
		asset_1: AssetId,
		amount_0_desired: AssetBalance,
		amount_1_desired: AssetBalance,
		amount_0_min: AssetBalance,
		amount_1_min: AssetBalance,
		at: Option<BlockHash>,
	) -> Result<NumberOrHex>;

	#[rpc(name = "zenlinkProtocol_hashesOfMaker")]
	fn hashes_of_maker(&self, maker: AccountId, page: u64, limit: u64, at: Option<BlockHash>) -> Result<Vec<H256>>;

	#[rpc(name = "zenlinkProtocol_hashesOfMakerInvert")]
	fn hashes_of_maker_invert(
		&self,
		maker: AccountId,
		page: u64,
		limit: u64,
		at: Option<BlockHash>,
	) -> Result<Vec<H256>>;

	#[rpc(name = "zenlinkProtocol_hashesOfFromToken")]
	fn hashes_of_from_token(
		&self,
		from_token: AssetId,
		page: u64,
		limit: u64,
		at: Option<BlockHash>,
	) -> Result<Vec<H256>>;

	#[rpc(name = "zenlinkProtocol_hashesOfToToken")]
	fn hashes_of_to_token(&self, to_token: AssetId, page: u64, limit: u64, at: Option<BlockHash>) -> Result<Vec<H256>>;

	#[rpc(name = "zenlinkProtocol_allHashes")]
	fn all_hashes(&self, page: u64, limit: u64, at: Option<BlockHash>) -> Result<Vec<H256>>;

	#[rpc(name = "zenlinkProtocol_numberOfHashesOfMaker")]
	fn number_of_hashes_of_maker(&self, maker: AccountId, at: Option<BlockHash>) -> Result<u64>;

	#[rpc(name = "zenlinkProtocol_numberOfHashesOfFromToken")]
	fn number_of_hashes_of_from_token(&self, from_token: AssetId, at: Option<BlockHash>) -> Result<u64>;

	#[rpc(name = "zenlinkProtocol_numberOfHashesOfToToken")]
	fn number_of_hashes_of_to_token(&self, to_token: AssetId, at: Option<BlockHash>) -> Result<u64>;

	#[rpc(name = "zenlinkProtocol_numberOfAllHashes")]
	fn number_of_all_hashes(&self, at: Option<BlockHash>) -> Result<u64>;
}

const RUNTIME_ERROR: i64 = 1;

pub struct ZenlinkProtocol<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> ZenlinkProtocol<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

impl<C, Block, AccountId> ZenlinkProtocolApi<<Block as BlockT>::Hash, AccountId> for ZenlinkProtocol<C, Block>
where
	Block: BlockT,
	AccountId: Codec,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: ZenlinkProtocolRuntimeApi<Block, AccountId>,
{
	fn get_balance(
		&self,
		asset_id: AssetId,
		account: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_balance(&at, asset_id, account)
			.map(|asset_balance| asset_balance.into())
			.map_err(runtime_error_into_rpc_err)
	}

	fn get_sovereigns_info(
		&self,
		asset_id: AssetId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(u32, AccountId, NumberOrHex)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_sovereigns_info(&at, asset_id)
			.map(|infos| {
				infos
					.into_iter()
					.map(|(para_id, account, asset_balance)| (para_id, account, asset_balance.into()))
					.collect::<Vec<_>>()
			})
			.map_err(runtime_error_into_rpc_err)
	}

	fn get_pair_by_asset_id(
		&self,
		asset_0: AssetId,
		asset_1: AssetId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Option<PairInfo<AccountId, NumberOrHex>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_pair_by_asset_id(&at, asset_0, asset_1)
			.map(|pairs| {
				pairs.map(|pair| PairInfo {
					asset_0: pair.asset_0,
					asset_1: pair.asset_1,
					account: pair.account,
					total_liquidity: pair.total_liquidity.into(),
					holding_liquidity: pair.holding_liquidity.into(),
					reserve_0: pair.reserve_0.into(),
					reserve_1: pair.reserve_1.into(),
					lp_asset_id: pair.lp_asset_id,
					status: pair.status,
				})
			})
			.map_err(runtime_error_into_rpc_err)
	}

	//buy amount asset price
	fn get_amount_in_price(
		&self,
		supply: AssetBalance,
		path: Vec<AssetId>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_amount_in_price(&at, supply, path)
			.map(|price| price.into())
			.map_err(runtime_error_into_rpc_err)
	}

	//sell amount asset price
	fn get_amount_out_price(
		&self,
		supply: AssetBalance,
		path: Vec<AssetId>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_amount_out_price(&at, supply, path)
			.map(|price| price.into())
			.map_err(runtime_error_into_rpc_err)
	}

	fn get_estimate_lptoken(
		&self,
		asset_0: AssetId,
		asset_1: AssetId,
		amount_0_desired: AssetBalance,
		amount_1_desired: AssetBalance,
		amount_0_min: AssetBalance,
		amount_1_min: AssetBalance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_estimate_lptoken(
			&at,
			asset_0,
			asset_1,
			amount_0_desired,
			amount_1_desired,
			amount_0_min,
			amount_1_min,
		)
		.map(|price| price.into())
		.map_err(runtime_error_into_rpc_err)
	}

	fn hashes_of_maker(
		&self,
		maker: AccountId,
		page: u64,
		limit: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<H256>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.hashes_of_maker(&at, maker, page, limit)
			.map_err(runtime_error_into_rpc_err)
	}

	fn hashes_of_maker_invert(
		&self,
		maker: AccountId,
		page: u64,
		limit: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<H256>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.hashes_of_maker_invert(&at, maker, page, limit)
			.map_err(runtime_error_into_rpc_err)
	}

	fn hashes_of_from_token(
		&self,
		from_token: AssetId,
		page: u64,
		limit: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<H256>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.hashes_of_from_token(&at, from_token, page, limit)
			.map_err(runtime_error_into_rpc_err)
	}

	fn hashes_of_to_token(
		&self,
		to_token: AssetId,
		page: u64,
		limit: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<H256>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.hashes_of_to_token(&at, to_token, page, limit)
			.map_err(runtime_error_into_rpc_err)
	}

	fn all_hashes(&self, page: u64, limit: u64, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<H256>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.all_hashes(&at, page, limit).map_err(runtime_error_into_rpc_err)
	}

	fn number_of_hashes_of_maker(&self, maker: AccountId, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.number_of_hashes_of_maker(&at, maker)
			.map_err(runtime_error_into_rpc_err)
	}

	fn number_of_hashes_of_from_token(&self, from_token: AssetId, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.number_of_hashes_of_from_token(&at, from_token)
			.map_err(runtime_error_into_rpc_err)
	}

	fn number_of_hashes_of_to_token(&self, to_token: AssetId, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.number_of_hashes_of_to_token(&at, to_token)
			.map_err(runtime_error_into_rpc_err)
	}

	fn number_of_all_hashes(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		api.number_of_all_hashes(&at).map_err(runtime_error_into_rpc_err)
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
