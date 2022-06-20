// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! RPC interface for the stable amm pallet.
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::traits::MaybeDisplay;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

use stable_amm_runtime_api::StableAmmApi as StableAmmRuntimeApi;

#[rpc]
pub trait StableAmmApi<BlockHash, CurrencyId, Balance, AccountId, PoolId> {
	#[rpc(name = "stable_amm_get_virtual_price")]
	fn get_virtual_price(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<NumberOrHex>;

	#[rpc(name = "stable_amm_get_A")]
	fn get_a(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<NumberOrHex>;

	#[rpc(name = "stable_amm_get_A_precise")]
	fn get_a_precise(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<NumberOrHex>;

	#[rpc(name = "stable_amm_get_currencies")]
	fn get_currencies(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<Vec<CurrencyId>>;

	#[rpc(name = "stable_amm_get_currencies")]
	fn get_currency(&self, pool_id: PoolId, index: u32, at: Option<BlockHash>) -> Result<CurrencyId>;

	#[rpc(name = "stable_amm_get_lp_currency")]
	fn get_lp_currency(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<CurrencyId>;

	#[rpc(name = "stable_amm_get_currency_index")]
	fn get_currency_index(&self, pool_id: PoolId, currency: CurrencyId, at: Option<BlockHash>) -> Result<u32>;

	#[rpc(name = "stable_amm_get_currency_precision_multipliers")]
	fn get_currency_precision_multipliers(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<Vec<NumberOrHex>>;

	#[rpc(name = "stable_amm_get_currency_balances")]
	fn get_currency_balances(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<Vec<NumberOrHex>>;

	#[rpc(name = "stable_amm_get_number_of_currencies")]
	fn get_number_of_currencies(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<u32>;

	#[rpc(name = "stable_amm_get_admin_balances")]
	fn get_admin_balances(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<Vec<NumberOrHex>>;

	#[rpc(name = "stable_amm_get_admin_balance")]
	fn get_admin_balance(&self, pool_id: PoolId, index: u32, at: Option<BlockHash>) -> Result<NumberOrHex>;

	#[rpc(name = "stable_amm_calculate_currency_amount")]
	fn calculate_currency_amount(
		&self,
		pool_id: PoolId,
		amounts: Vec<Balance>,
		deposit: bool,
		at: Option<BlockHash>,
	) -> Result<NumberOrHex>;

	#[rpc(name = "stable_amm_calculate_swap")]
	fn calculate_swap(
		&self,
		pool_id: PoolId,
		in_index: u32,
		out_index: u32,
		in_amount: Balance,
		at: Option<BlockHash>,
	) -> Result<NumberOrHex>;

	#[rpc(name = "stable_amm_calculate_remove_liquidity")]
	fn calculate_remove_liquidity(
		&self,
		pool_id: PoolId,
		amount: Balance,
		at: Option<BlockHash>,
	) -> Result<Vec<NumberOrHex>>;

	#[rpc(name = "stable_amm_calculate_calculate_remove_liquidity_one_currency")]
	fn calculate_remove_liquidity_one_currency(
		&self,
		pool_id: PoolId,
		amount: Balance,
		index: u32,
		at: Option<BlockHash>,
	) -> Result<NumberOrHex>;
}

pub struct StableAmm<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> StableAmm<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

impl<C, Block, CurrencyId, Balance, AccountId, PoolId>
	StableAmmApi<<Block as BlockT>::Hash, CurrencyId, Balance, AccountId, PoolId> for StableAmm<C, Block>
where
	Block: BlockT,
	CurrencyId: Codec + std::cmp::PartialEq,
	Balance: Codec + TryInto<NumberOrHex> + std::fmt::Debug + MaybeDisplay + Copy,
	AccountId: Codec,
	PoolId: Codec,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: StableAmmRuntimeApi<Block, CurrencyId, Balance, AccountId, PoolId>,
{
	fn get_virtual_price(&self, pool_id: PoolId, at: Option<<Block as BlockT>::Hash>) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let price = api
			.get_virtual_price(&at, pool_id)
			.map_err(runtime_error_into_rpc_err)?;

		try_into_rpc_balance(price)
	}

	fn get_a(&self, pool_id: PoolId, at: Option<<Block as BlockT>::Hash>) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let price = api.get_a(&at, pool_id).map_err(runtime_error_into_rpc_err)?;

		try_into_rpc_balance(price)
	}

	fn get_a_precise(&self, pool_id: PoolId, at: Option<<Block as BlockT>::Hash>) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let price = api.get_a_precise(&at, pool_id).map_err(runtime_error_into_rpc_err)?;

		try_into_rpc_balance(price)
	}

	fn get_currencies(&self, pool_id: PoolId, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<CurrencyId>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_currencies(&at, pool_id).map_err(runtime_error_into_rpc_err)
	}

	fn get_currency(&self, pool_id: PoolId, index: u32, at: Option<<Block as BlockT>::Hash>) -> Result<CurrencyId> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_currency(&at, pool_id, index).map_or_else(
			|e| Err(runtime_error_into_rpc_err(e)),
			|v| v.ok_or(runtime_error_into_rpc_err("not found")),
		)
	}

	fn get_lp_currency(&self, pool_id: PoolId, at: Option<<Block as BlockT>::Hash>) -> Result<CurrencyId> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_lp_currency(&at, pool_id).map_or_else(
			|e| Err(runtime_error_into_rpc_err(e)),
			|v| v.ok_or(runtime_error_into_rpc_err("not found")),
		)
	}

	fn get_currency_index(
		&self,
		pool_id: PoolId,
		currency: CurrencyId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<u32> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let currencies = api.get_currencies(&at, pool_id).map_err(runtime_error_into_rpc_err)?;

		for (i, c) in currencies.iter().enumerate() {
			if *c == currency {
				return Ok(i as u32);
			}
		}
		Err(runtime_error_into_rpc_err("invalid index"))
	}

	fn get_currency_precision_multipliers(
		&self,
		pool_id: PoolId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<NumberOrHex>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_currency_precision_multipliers(&at, pool_id)
			.map_err(runtime_error_into_rpc_err)?
			.iter()
			.map(|b| try_into_rpc_balance(*b))
			.collect()
	}

	fn get_currency_balances(&self, pool_id: PoolId, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<NumberOrHex>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_currency_balances(&at, pool_id)
			.map_err(runtime_error_into_rpc_err)?
			.iter()
			.map(|b| try_into_rpc_balance(*b))
			.collect()
	}

	fn get_number_of_currencies(&self, pool_id: PoolId, at: Option<<Block as BlockT>::Hash>) -> Result<u32> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_number_of_currencies(&at, pool_id)
			.map_err(runtime_error_into_rpc_err)
	}

	fn get_admin_balances(&self, pool_id: PoolId, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<NumberOrHex>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.get_admin_balances(&at, pool_id)
			.map_err(runtime_error_into_rpc_err)?
			.iter()
			.map(|b| try_into_rpc_balance(*b))
			.collect()
	}

	fn get_admin_balance(
		&self,
		pool_id: PoolId,
		index: u32,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let balances = api
			.get_admin_balances(&at, pool_id)
			.map_err(runtime_error_into_rpc_err)?;

		for (i, balance) in balances.iter().enumerate() {
			if i as u32 == index {
				return try_into_rpc_balance(*balance);
			}
		}

		Err(runtime_error_into_rpc_err("invalid index"))
	}

	fn calculate_currency_amount(
		&self,
		pool_id: PoolId,
		amounts: Vec<Balance>,
		deposit: bool,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let amount = api
			.calculate_currency_amount(&at, pool_id, amounts, deposit)
			.map_err(runtime_error_into_rpc_err)?;

		try_into_rpc_balance(amount)
	}

	fn calculate_swap(
		&self,
		pool_id: PoolId,
		in_index: u32,
		out_index: u32,
		in_amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let amount = api
			.calculate_swap(&at, pool_id, in_index, out_index, in_amount)
			.map_err(runtime_error_into_rpc_err)?;

		try_into_rpc_balance(amount)
	}

	fn calculate_remove_liquidity(
		&self,
		pool_id: PoolId,
		amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<NumberOrHex>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		api.calculate_remove_liquidity(&at, pool_id, amount)
			.map_err(runtime_error_into_rpc_err)?
			.iter()
			.map(|b| try_into_rpc_balance(*b))
			.collect()
	}

	fn calculate_remove_liquidity_one_currency(
		&self,
		pool_id: PoolId,
		amount: Balance,
		index: u32,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<NumberOrHex> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

		let amount = api
			.calculate_remove_liquidity_one_currency(&at, pool_id, amount, index)
			.map_err(runtime_error_into_rpc_err)?;

		try_into_rpc_balance(amount)
	}
}

fn try_into_rpc_balance<Balance: Codec + TryInto<NumberOrHex> + MaybeDisplay + Copy + std::fmt::Debug>(
	value: Balance,
) -> Result<NumberOrHex> {
	value.try_into().map_err(|_| RpcError {
		code: ErrorCode::InvalidParams,
		message: format!("{:#?} doesn't fit in NumberOrHex representation", value),
		data: None,
	})
}

fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> RpcError {
	RpcError {
		code: ErrorCode::ServerError(1),
		message: "Stable Amm trapped".into(),
		data: Some(format!("{:?}", err).into()),
	}
}
