// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! # OrderBook Module
//!
//! ## Overview
//!
//! Built-in decentralized limit order book modules in Substrate network.

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use super::*;
use sp_core::{sr25519, H256, U256};

use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Keccak256, Verify},
	AnySignature,
};

/// Limit order
#[derive(Encode, Decode, Default, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct LimitOrder<BlockNumber, AccountId> {
	pub maker: AccountId,
	pub from_asset_id: AssetId,
	pub to_asset_id: AssetId,
	pub amount_in: u128,
	pub amount_out_min: u128,
	pub recipient: AccountId,
	pub deadline: BlockNumber,
	pub create_at: u64,
	pub signature: Vec<u8>,
}

#[derive(Encode, Decode, Default, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct FillOrderArgs<BlockNumber, AccountId> {
	pub order: LimitOrder<BlockNumber, AccountId>,
	pub amount_to_fill_in: u128,
	pub path: Vec<AssetId>,
}

impl<BlockNumber, AccountId> LimitOrder<BlockNumber, AccountId>
where
	AccountId: Parameter + Default,
	BlockNumber: Parameter + Default,
{
	pub fn hash(&mut self) -> Result<H256, DispatchError> {
		let mut data = Vec::from(account_to_bytes(&self.maker)?);
		data.append(&mut self.from_asset_id.encode());
		data.append(&mut self.to_asset_id.encode());
		data.append(&mut self.amount_in.encode());
		data.append(&mut self.amount_out_min.encode());
		data.append(&mut Vec::from(account_to_bytes(&self.recipient)?));
		data.append(&mut self.deadline.encode());

		Ok(Keccak256::hash(&data))
	}

	pub fn validate(&self) -> Result<(), DispatchError> {
		ensure!(self.maker != AccountId::default(), "invalid maker");
		ensure!(self.from_asset_id != AssetId::default(), "invalid from_asset_id");
		ensure!(self.to_asset_id != AssetId::default(), "invalid to_asset_id");
		ensure!(self.to_asset_id != self.from_asset_id, "duplicate-tokens");
		ensure!(self.amount_in > 0u128, "invalid amount_in");
		ensure!(self.amount_out_min > 0u128, "invalid amount_out_min");
		ensure!(self.recipient != AccountId::default(), "invalid recipient");
		ensure!(self.deadline != BlockNumber::default(), "invalid deadline");
		Ok(())
	}
}

fn account_to_bytes<AccountId>(account: &AccountId) -> Result<[u8; 32], DispatchError>
where
	AccountId: Encode,
{
	let account_vec = account.encode();
	ensure!(account_vec.len() == 32, "AccountId must be 32 bytes.");
	let mut bytes = [0u8; 32];
	bytes.copy_from_slice(&account_vec);
	Ok(bytes)
}

impl<T: Config> Pallet<T> {
	// DomainSeparator = keccak256({name, version, chainId, pallet_account})
	fn domain_separator() -> H256 {
		let mut name = Vec::from("OrderBook");
		let mut chain_id = Vec::from(T::SelfParaId::get().to_be_bytes());
		let mut version = Vec::from(1u32.to_be_bytes());
		let account: T::AccountId = T::PalletId::get().into_account();

		name.append(&mut version);
		name.append(&mut chain_id);
		name.append(&mut account.encode());

		Keccak256::hash(&name)
	}

	pub(crate) fn inner_create_order(order: &mut LimitOrder<T::BlockNumber, T::AccountId>) -> DispatchResult {
		order.validate()?;
		let order_hash = order.hash().map_err(|_| Error::<T>::ExistRewardsInBootstrap)?;

		let separator = Self::domain_separator();

		let mut msg = Vec::<u8>::from(separator.as_bytes());
		msg.append(&mut Vec::<u8>::from(order_hash.as_bytes()));

		let digest = Keccak256::hash(&msg);

		Self::verify_signature(&order.maker, digest.as_bytes(), &order.signature)?;
		ensure!(
			Self::get_order_of_hash(order_hash).maker == T::AccountId::default(),
			Error::<T>::LimitOrderAlreadyExist
		);

		AllOrderHashes::<T>::mutate(|hashes| hashes.push(order_hash));
		HashesOfMaker::<T>::mutate(&order.maker, |hashes| hashes.push(order_hash));
		HashesOfFromAssetId::<T>::mutate(order.from_asset_id, |hashes| hashes.push(order_hash));
		HashesOfToAssetId::<T>::mutate(order.to_asset_id, |hashes| hashes.push(order_hash));
		OrderOfHash::<T>::insert(order_hash, order);

		Self::deposit_event(Event::OrderCreate(order_hash));
		Ok(())
	}

	pub(crate) fn inner_fill_order(args: &mut FillOrderArgs<T::BlockNumber, T::AccountId>) -> DispatchResult {
		let order_hash = args.order.hash()?;
		Self::validate_order_status(args, order_hash)?;

		let separator = Self::domain_separator();

		let mut msg = Vec::<u8>::from(separator.as_bytes());
		msg.append(&mut Vec::<u8>::from(order_hash.as_bytes()));

		let digest = Keccak256::hash(&msg);
		Self::verify_signature(&args.order.maker, digest.as_bytes(), &args.order.signature)?;

		let amount_out_min = U256::from(args.order.amount_out_min)
			.saturating_mul(U256::from(args.amount_to_fill_in))
			.checked_div(U256::from(args.order.amount_in))
			.and_then(|n| TryInto::<AssetBalance>::try_into(n).ok())
			.ok_or(Error::<T>::Overflow)?;

		let swap_amount_out = Self::inner_swap_exact_assets_for_assets(
			&args.order.maker,
			args.amount_to_fill_in,
			amount_out_min,
			&*args.path,
			&args.order.recipient,
		)?;

		FilledAmountInOfHash::<T>::try_mutate(order_hash, |filled_amount_in| -> DispatchResult {
			*filled_amount_in = filled_amount_in
				.checked_add(args.amount_to_fill_in)
				.ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		FilledAmountOutOfHash::<T>::try_mutate(order_hash, |filled_amount_out| -> DispatchResult {
			*filled_amount_out = filled_amount_out
				.checked_add(swap_amount_out)
				.ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		Self::deposit_event(Event::OrderFilled(
			order_hash,
			args.order.recipient.clone(),
			swap_amount_out,
		));

		Ok(())
	}

	pub(crate) fn inner_cancel_order(who: &T::AccountId, order_hash: H256) -> DispatchResult {
		ensure!(
			Self::get_order_of_hash(order_hash) != LimitOrder::<T::BlockNumber, T::AccountId>::default(),
			"order not exist"
		);
		CanceledOfHash::<T>::mutate(who, |cancel_tree| cancel_tree.insert(order_hash, true));
		Ok(())
	}

	fn verify_signature(who: &T::AccountId, message: &[u8], signature: &[u8]) -> DispatchResult {
		// sr25519 always expects a 64 byte signature.
		ensure!(signature.len() == 64, Error::<T>::InvalidSignature);

		let signature: AnySignature = sr25519::Signature::from_slice(signature).into();

		let account_bytes: [u8; 32] = account_to_bytes(who)?;
		let public_key = sr25519::Public::from_raw(account_bytes);

		// Check if everything is good or not.
		match signature.verify(message, &public_key) {
			true => Ok(()),
			false => Err(Error::<T>::InvalidSignature)?,
		}
	}

	fn validate_order_status(
		args: &mut FillOrderArgs<T::BlockNumber, T::AccountId>,
		order_hash: H256,
	) -> DispatchResult {
		ensure!(
			Self::get_order_of_hash(order_hash) != LimitOrder::<T::BlockNumber, T::AccountId>::default(),
			"order not exist"
		);
		ensure!(
			args.order.deadline >= frame_system::Pallet::<T>::block_number(),
			"order expire"
		);

		ensure!(
			CanceledOfHash::<T>::get(&args.order.maker).get(&order_hash) == None,
			"order canceled"
		);

		let total_fill_in = Self::get_amount_fill_in_of_hash(order_hash)
			.checked_add(args.amount_to_fill_in)
			.ok_or(Error::<T>::Overflow)?;

		ensure!(total_fill_in <= args.order.amount_in, "order already filled");

		Ok(())
	}
}
