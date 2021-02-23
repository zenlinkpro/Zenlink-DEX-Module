// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod assets;
mod primitives;
mod rpc;
mod swap;
mod xcm_support;
mod xtransfer;

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, Get},
};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{Convert, StaticLookup},
	ModuleId,
};
use sp_std::{vec, prelude::Vec};

pub use crate::{
	primitives::{AssetId, MultiAsset as ZenlinkMultiAsset, PairId, TokenBalance},
	rpc::PairInfo,
	swap::Pair,
	xcm_support::{ParaChainWhiteList, Transactor},
	xtransfer::Origin,
};
pub use cumulus_primitives_core::{
	relay_chain::Balance as RelayChainBalance, DownwardMessageHandler, HrmpMessageHandler,
	HrmpMessageSender, InboundDownwardMessage, InboundHrmpMessage, OutboundHrmpMessage, ParaId,
	UpwardMessage, UpwardMessageSender,
};
pub use polkadot_parachain::primitives::Sibling;
pub use xcm::{
	v0::{
		Error as XcmError, ExecuteXcm, Junction, MultiAsset, MultiLocation, NetworkId, Order,
		Result as XcmResult, SendXcm, Xcm,
	},
	VersionedXcm,
};
pub use xcm_builder::{
	AccountId32Aliases, LocationInverter, ParentIsDefault, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SovereignSignedViaLocation,
};
pub use xcm_executor::{
	traits::{FilterAssetLocation, LocationConversion, TransactAsset},
	Config as XcmCfg, XcmExecutor,
};

pub trait Config: frame_system::Config {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	/// This chain native currency.
	type NativeCurrency: Currency<Self::AccountId>;
	/// Something to execute an XCM message.
	type XcmExecutor: ExecuteXcm;
	/// Something to send an upward message.
	type UpwardMessageSender: UpwardMessageSender;
	/// Something to send an HRMP message.
	type HrmpMessageSender: HrmpMessageSender;

	type ModuleId: Get<ModuleId>;
	type AccountIdConverter: LocationConversion<Self::AccountId>;
	type AccountId32Converter: Convert<Self::AccountId, [u8; 32]>;
	type ParaId: Get<ParaId>;
}

decl_storage! {
	trait Store for Module<T: Config> as ZenlinkProtocol {
		/// The number of units of assets held by any given account.
		Balances: map hasher(blake2_128_concat) (AssetId, T::AccountId) => TokenBalance;

		/// TWOX-NOTE: `AssetId` is trusted, so this is safe.
		TotalSupply: map hasher(twox_64_concat) AssetId => TokenBalance;

		/// The assets list
		Assets: Vec<AssetId>;

		AssetsToPair get(fn tokens_to_pair): map hasher(blake2_128_concat) (AssetId, AssetId) => Option<Pair<T::AccountId, TokenBalance>>;

		Pairs: Vec<(AssetId, AssetId)>;

		NextPairId get(fn next_pair_id): PairId;

		LiquidityPool get(fn get_liquidity): map hasher(blake2_128_concat) (T::AccountId, T::AccountId) =>TokenBalance;
	}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
		<T as frame_system::Config>::Hash
	{
		/// Assets

		/// Some assets were transferred. \[asset_id, owner, target, amount\]
		Transferred(AssetId, AccountId, AccountId, TokenBalance),
		/// Some assets were burned. \[asset_id, owner, amount\]
		Burned(AssetId, AccountId, TokenBalance),
		/// Some assets were minted. \[asset_id, owner, amount\]
		Minted(AssetId, AccountId, TokenBalance),

		/// Xtransfer

		/// Transferred to parachain. [asset_id, src, para_id, dest, amount]
		TransferredToParachain(AssetId, AccountId, ParaId, AccountId, TokenBalance),
		/// Some XCM was executed ok.
		Success(Hash),
		/// Some XCM failed.
		Fail(Hash, XcmError),
		/// Bad XCM version used.
		BadVersion(Hash),
		/// Bad XCM format used.
		BadFormat(Hash),
		/// An upward message was sent to the relay chain.
		UpwardMessageSent(Hash),
		/// An HRMP message was sent to a sibling parachainchain.
		HrmpMessageSent(Hash),

		/// Create a trading pair
		PairCreated(AccountId, AssetId, AssetId),
		/// Add liquidity
		LiquidityAdded(AccountId, AssetId, AssetId),
		/// Withdraw liquidity
		LiquidityRemoved(AccountId, AccountId, AssetId, AssetId, TokenBalance),
		/// Transact in trading
		TokenSwap(AccountId, AccountId, Vec<AssetId>),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Account balance must be greater than or equal to the transfer amount.
		InsufficientAssetBalance,
		/// Asset has not been created.
		AssetNotExists,
		/// AssetId is native currency
		NotParaCurrency,

		/// Location given was invalid or unsupported.
		AccountIdBadLocation,
		/// XCM execution failed
		ExecutionFailed,
		/// Transfer to self by XCM message
		DeniedTransferToSelf,

		/// Trading pair can't be created.
		DeniedCreatePair,
		/// Trading pair already exists.
		PairAlreadyExists,
		/// Trading pair does not exist.
		PairNotExists,
		/// Swap in local parachain by XCM message
		DeniedSwapInLocal,
		/// Liquidity is not enough.
		InsufficientLiquidity,
		/// Trading pair does have enough asset.
		InsufficientPairReserve,
		/// Get target amount is less than exception.
		InsufficientTargetAmount,
		/// Sold amount is more than exception.
		ExcessiveSoldAmount,
		/// Can't find pair though trading path.
		InvalidPath,
		/// Ensure correct parameter in cross chain add liquidity.
		DeniedAddLiquidityToParachain,
		/// Incorrect asset amount range.
		IncorrectAssetAmountRange,
		/// Overflow.
		Overflow,
		/// Transaction block number is larger than the end block number.
		Deadline,
	}
}
// TODO: weight
// TODO: transactional
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
		/// Move some assets from one holder to another.
		///
		/// - `asset_id`: the asset id.
		/// - `target`: the receiver of the asset.
		/// - `amount`: the amount of the asset to transfer.
		#[weight = 0]
		fn transfer(
			origin,
			asset_id: AssetId,
			target: <T::Lookup as StaticLookup>::Source,
			amount: TokenBalance
		) -> DispatchResult {
			ensure!(asset_id.is_para_currency(), Error::<T>::NotParaCurrency);
			let origin = ensure_signed(origin)?;
			let target = T::Lookup::lookup(target)?;

			Self::inner_transfer(asset_id, &origin, &target, amount)?;

			Ok(())
		}

		/// Transfer zenlink assets to a sibling parachain.
		///
		/// Zenlink assets can be either native or foreign to the sending parachain.
		///
		/// # Arguments
		///
		/// * `asset_id`: Global identifier for a zenlink asset
		/// * `para_id`: Destination parachain
		/// * `account`: Destination account
		/// * `amount`: Amount to transfer
		#[weight = 10]
		pub fn transfer_to_parachain(
			origin,
			asset_id: AssetId,
			para_id: ParaId,
			account: T::AccountId,
			amount: TokenBalance
		) -> DispatchResult {
			ensure!(para_id != T::ParaId::get(), Error::<T>::DeniedTransferToSelf);
			let who = ensure_signed(origin)?;
			let xcm = Self::make_xcm_transfer_to_parachain(&asset_id, para_id, &account, amount);

			let xcm_origin = T::AccountIdConverter::try_into_location(who.clone())
				.map_err(|_| Error::<T>::AccountIdBadLocation)?;

			T::XcmExecutor::execute_xcm(xcm_origin, xcm)
				.map_err(|err| {
					frame_support::debug::print!("zenlink::<transfer_to_parachain>: err = {:?}", err);
					Error::<T>::ExecutionFailed
				})?;

			Self::deposit_event(
				Event::<T>::TransferredToParachain(asset_id, who, para_id, account, amount),
			);

			Ok(())
		}

		#[weight = 10]
		pub fn create_pair(
			origin,
			token_0: AssetId,
			token_1: AssetId,
		 ) -> DispatchResult {
			 let _who = ensure_signed(origin)?;
			Self::inner_create_pair(&token_0, &token_1)?;
			Ok(())
		}

		#[weight = 0]
		#[allow(clippy::too_many_arguments)]
		pub fn add_liquidity(
			origin,
			token_0: AssetId,
			token_1: AssetId,
			amount_0_desired : TokenBalance,
			amount_1_desired : TokenBalance,
			amount_0_min : TokenBalance,
			amount_1_min : TokenBalance,
			target_parachain: ParaId,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let now = frame_system::Module::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);
			let who = ensure_signed(origin)?;

			Self::inner_add_liquidity(&who, &token_0, &token_1, amount_0_desired, amount_1_desired, amount_0_min, amount_1_min)?;
			Self::deposit_event(RawEvent::LiquidityAdded(who, token_0, token_1));
			Ok(())
		}

		#[weight = 0]
		#[allow(clippy::too_many_arguments)]
		pub fn remove_liquidity(
			origin,
			token_0: AssetId,
			token_1: AssetId,
			liquidity: TokenBalance,
			amount_token_0_min : TokenBalance,
			amount_token_1_min : TokenBalance,
			to: <T::Lookup as StaticLookup>::Source,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let now = frame_system::Module::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			Self::inner_remove_liquidity(&who, &token_0, &token_1, liquidity, amount_token_0_min, amount_token_1_min, &to)?;
			Self::deposit_event(RawEvent::LiquidityRemoved(who, to, token_0, token_1, liquidity));
			Ok(())
		}

		#[weight = 0]
		pub fn swap_exact_tokens_for_tokens(
			origin,
			amount_in: TokenBalance,
			amount_out_min: TokenBalance,
			path: Vec<AssetId>,
			to: <T::Lookup as StaticLookup>::Source,
			target_parachain: ParaId,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let now = frame_system::Module::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			Self::inner_swap_exact_tokens_for_tokens(&who, amount_in, amount_out_min, &path, &to)?;

			Self::deposit_event(RawEvent::TokenSwap(who, to, path));
			Ok(())
		}

		#[weight = 0]
		pub fn swap_tokens_for_exact_tokens(
			origin,
			amount_out: TokenBalance,
			amount_in_max: TokenBalance,
			path: Vec<AssetId>,
			to: <T::Lookup as StaticLookup>::Source,
			target_parachain: ParaId,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let now = frame_system::Module::<T>::block_number();
			ensure!(deadline > now, Error::<T>::Deadline);

			let who = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
			Self::inner_swap_tokens_for_exact_tokens(&who, amount_out, amount_in_max, &path, &to)?;

			Self::deposit_event(RawEvent::TokenSwap(who, to, path));
			Ok(())
		}
	}
}
