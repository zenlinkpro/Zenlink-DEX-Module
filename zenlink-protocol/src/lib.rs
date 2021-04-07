// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use cumulus_primitives_core::{
    relay_chain, relay_chain::Balance as RelayChainBalance, DownwardMessageHandler,
    InboundDownwardMessage, ParaId, ServiceQuality, UpwardMessage, UpwardMessageSender,
    XcmpMessageHandler, XcmpMessageSender,
};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    transactional,
};
use frame_system::ensure_signed;

use sp_runtime::{
    traits::{Convert, StaticLookup},
    ModuleId,
};
use sp_std::{convert::TryInto, prelude::Vec, vec};

pub use xcm::v0::{
    Error as XcmError, ExecuteXcm, Junction, MultiAsset, MultiLocation, NetworkId, Order,
    Result as XcmResult, Xcm,
};

pub use xcm_executor::{
    traits::{FilterAssetLocation, LocationConversion, TransactAsset},
    XcmExecutor,
};

pub use crate::{
    primitives::{
        AssetId, AssetProperty, PairId, TokenBalance, INNER_ASSET, NATIVE_CURRENCY, OTHER_ASSET,
    },
    rpc::PairInfo,
    swap::Pair,
    traits::{AssetHandler, MultiAssetHandler, NativeCurrencyAdaptor},
    transactor::{ParaChainWhiteList, TransactorAdaptor},
};

mod assets;
mod primitives;
mod rpc;
mod swap;
mod traits;
mod transactor;

const LOG_TARGET: &str = "zenlink_protocol";

pub fn make_x2_location(para_id: u32) -> MultiLocation {
    MultiLocation::X2(Junction::Parent, Junction::Parachain { id: para_id })
}

pub trait Config: frame_system::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    /// Something to execute an XCM message.
    type XcmExecutor: ExecuteXcm;
    /// The set of parachains which the xcm can reach.
    type TargetChains: Get<Vec<(MultiLocation, u128)>>;
    /// The Zenlink Protocol Module Id.
    type ModuleId: Get<ModuleId>;
    /// Convert AccountId to MultiLocation.
    type AccountIdConverter: LocationConversion<Self::AccountId>;
    /// Convert AccountId to [u8; 32].
    type AccountId32Converter: Convert<Self::AccountId, [u8; 32]>;
    /// Get this parachain Id.
    type ParaId: Get<ParaId>;
    /// This chain native currency.
    type NativeCurrency: AssetHandler<Self::AccountId>;
    /// This chain other assets.
    type OtherAssets: AssetHandler<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Config> as ZenlinkProtocol {
        /// The number of units of assets held by any given account.
        Balances: map hasher(blake2_128_concat) (AssetId, T::AccountId) => TokenBalance;

        /// TWOX-NOTE: `AssetId` is trusted, so this is safe.
        TotalSupply: map hasher(twox_64_concat) AssetId => TokenBalance;

        AssetsMetadata get(fn asset_property): map hasher(blake2_128_concat) AssetId => AssetProperty;

        /// The assets list
        Assets: Vec<AssetId>;

        AssetsToPair get(fn tokens_to_pair): map hasher(blake2_128_concat) (AssetId, AssetId) => Option<Pair<T::AccountId, TokenBalance>>;

        Pairs: Vec<(AssetId, AssetId)>;

        NextPairId get(fn next_pair_id): PairId;
    }
}

decl_event! {
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId
    {
        /// Assets

        /// Some assets were transferred. \[asset_id, owner, target, amount\]
        Transferred(AssetId, AccountId, AccountId, TokenBalance),
        /// Some assets were burned. \[asset_id, owner, amount\]
        Burned(AssetId, AccountId, TokenBalance),
        /// Some assets were minted. \[asset_id, owner, amount\]
        Minted(AssetId, AccountId, TokenBalance),
        /// Some assets were Issued. \[asset_id, \]
        Issued(AssetId),
        /// Transferred to parachain. \[asset_id, src, para_id, dest, amount\]
        TransferredToParachain(AssetId, AccountId, ParaId, AccountId, TokenBalance),

        /// Swap

        /// Create a trading pair. \[creator, asset_id, asset_id\]
        PairCreated(AccountId, AssetId, AssetId),
        /// Add liquidity. \[owner, asset_id, asset_id\]
        LiquidityAdded(AccountId, AssetId, AssetId),
        /// Remove liquidity. \[owner, receiver, asset_id, asset_id, amount\]
        LiquidityRemoved(AccountId, AccountId, AssetId, AssetId, TokenBalance),
        /// Transact in trading \[owner, receiver, swap_path\]
        TokenSwap(AccountId, AccountId, Vec<AssetId>),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Account balance must be greater than or equal to the transfer amount.
        InsufficientAssetBalance,
        /// Asset has not been created.
        AssetNotExists,
        /// Asset has already exist.
        AssetAlreadyExist,
        /// AssetId is not in zenlink module
        NotZenlinkAsset,
        /// Amount to Balance conversion failed
        AmountToBalanceConversionFailed,

        /// Location given was invalid or unsupported.
        AccountIdBadLocation,
        /// The target chain is not in whitelist.
        DeniedReachTargetChain,
        /// XCM execution failed
        ExecutionFailed,
        /// Transfer to self by XCM message
        DeniedTransferToSelf,
        /// Value too low to create account due to existential deposit
        ExistentialDeposit,
        /// Invalid Asset Id.
        InvalidAssetId,

        /// Trading pair can't be created.
        DeniedCreatePair,
        /// Trading pair already exists.
        PairAlreadyExists,
        /// Trading pair does not exist.
        PairNotExists,
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
        /// Incorrect asset amount range.
        IncorrectAssetAmountRange,
        /// Overflow.
        Overflow,
        /// Transaction block number is larger than the end block number.
        Deadline,
    }
}

// TODO: weight
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;
        /// Move some assets from one holder to another.
        ///
        /// # Arguments
        ///
        /// - `asset_id`: The asset id.
        /// - `target`: The receiver of the asset.
        /// - `amount`: The amount of the asset to transfer.
        #[weight = 0]
        fn transfer(
            origin,
            asset_id: AssetId,
            target: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: TokenBalance
        ) -> DispatchResult {
            let origin = ensure_signed(origin)?;
            let target = T::Lookup::lookup(target)?;

            Self::multi_asset_transfer(&asset_id, &origin, &target, amount)?;

            Ok(())
        }

        /// Transfer zenlink assets to a sibling parachain.
        ///
        /// Zenlink assets can be either native or foreign to the sending parachain.
        ///
        /// # Arguments
        ///
        /// - `asset_id`: Global identifier for a zenlink asset
        /// - `para_id`: Destination parachain
        /// - `account`: Destination account
        /// - `amount`: Amount to transfer
        /// - `force_transfer`: Ignore check destination parachain and minimum balance
        #[weight = 10]
        #[transactional]
        pub fn transfer_to_parachain(
            origin,
            asset_id: AssetId,
            para_id: ParaId,
            account: T::AccountId,
            #[compact] amount: TokenBalance,
            force_transfer: bool
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(asset_id.is_valid(), Error::<T>::InvalidAssetId);
            ensure!(para_id != T::ParaId::get(), Error::<T>::DeniedTransferToSelf);
            ensure!(force_transfer || Self::is_reachable(para_id), Error::<T>::DeniedReachTargetChain);
            ensure!(force_transfer || Self::must_more_than_minimum(asset_id, para_id, amount), Error::<T>::ExistentialDeposit);
            ensure!(Self::multi_asset_balance_of(&asset_id, &who) >= amount, Error::<T>::InsufficientAssetBalance);

            let xcm = Self::make_xcm_transfer_to_parachain(&asset_id, para_id, &account, amount)
                .map_err(|_| Error::<T>::NotZenlinkAsset)?;

            let xcm_origin = T::AccountIdConverter::try_into_location(who.clone())
                .map_err(|_| Error::<T>::AccountIdBadLocation)?;

            #[allow(unused_variables)]
            T::XcmExecutor::execute_xcm(xcm_origin, xcm)
                .map_err(|err| {
                    log::error!{
                        target: LOG_TARGET,
                        "transfer_to_parachain: xcm execution failded, err = {:?}",
                        err
                    }
                    Error::<T>::ExecutionFailed
                })?;

            Self::deposit_event(
                Event::<T>::TransferredToParachain(asset_id, who, para_id, account, amount),
            );

            Ok(())
        }

        /// Create pair by tow asset.
        ///
        /// The order of asset dot effect result.
        ///
        /// # Arguments
        ///
        /// - `token_0`: Token that make up Pair
        /// - `token_1`: Token that make up Pair
        #[weight = 10]
        pub fn create_pair(
            origin,
            token_0: AssetId,
            token_1: AssetId
         ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::inner_create_pair(&token_0, &token_1)?;

            Self::deposit_event(RawEvent::PairCreated(who, token_0, token_1));

            Ok(())
        }

        /// Provide liquidity to a pair.
        ///
        /// The order of asset dot effect result.
        ///
        /// # Arguments
        ///
        /// - `token_0`: Token that make up pair
        /// - `token_1`: Token that make up pair
        /// - `amount_0_desired`: Maximum amount of token_0 added to the pair
        /// - `amount_1_desired`: Maximum amount of token_1 added to the pair
        /// - `amount_0_min`: Minimum amount of token_0 added to the pair
        /// - `amount_1_min`: Minimum amount of token_1 added to the pair
        /// - `deadline`: Height of the cutoff block of this transaction
        #[weight = 0]
        #[transactional]
        #[allow(clippy::too_many_arguments)]
        pub fn add_liquidity(
            origin,
            token_0: AssetId,
            token_1: AssetId,
            #[compact] amount_0_desired : TokenBalance,
            #[compact] amount_1_desired : TokenBalance,
            #[compact] amount_0_min : TokenBalance,
            #[compact] amount_1_min : TokenBalance,
            #[compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_add_liquidity(&who, &token_0, &token_1, amount_0_desired, amount_1_desired, amount_0_min, amount_1_min)?;

            Self::deposit_event(RawEvent::LiquidityAdded(who, token_0, token_1));

            Ok(())
        }

        /// Extract liquidity.
        ///
        /// The order of asset dot effect result.
        ///
        /// # Arguments
        ///
        /// - `token_0`: Token that make up pair
        /// - `token_1`: Token that make up pair
        /// - `amount_token_0_min`: Minimum amount of token_0 to exact
        /// - `amount_token_1_min`: Minimum amount of token_1 to exact
        /// - `to`: Account that accepts withdrawal of assets
        /// - `deadline`: Height of the cutoff block of this transaction
        #[weight = 0]
        #[transactional]
        #[allow(clippy::too_many_arguments)]
        pub fn remove_liquidity(
            origin,
            token_0: AssetId,
            token_1: AssetId,
            #[compact] liquidity: TokenBalance,
            #[compact] amount_token_0_min : TokenBalance,
            #[compact] amount_token_1_min : TokenBalance,
            to: <T::Lookup as StaticLookup>::Source,
            #[compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_remove_liquidity(&who, &token_0, &token_1, liquidity, amount_token_0_min, amount_token_1_min, &to)?;

            Self::deposit_event(RawEvent::LiquidityRemoved(who, to, token_0, token_1, liquidity));

            Ok(())
        }

        /// Sell amount of asset by path.
        ///
        /// # Arguments
        ///
        /// - `amount_in`: Amount asset will be sold
        /// - `amount_out_min`: Minimum amount of target asset
        /// - `path`: path can convert to pairs.
        /// - `to`: Account that receive the target asset
        /// - `deadline`: Height of the cutoff block of this transaction
        #[weight = 0]
        #[transactional]
        pub fn swap_exact_tokens_for_tokens(
            origin,
            #[compact] amount_in: TokenBalance,
            #[compact] amount_out_min: TokenBalance,
            path: Vec<AssetId>,
            to: <T::Lookup as StaticLookup>::Source,
            #[compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_swap_exact_tokens_for_tokens(&who, amount_in, amount_out_min, &path, &to)?;

            Self::deposit_event(RawEvent::TokenSwap(who, to, path));

            Ok(())
        }

        /// Buy amount of asset by path.
        ///
        /// # Arguments
        ///
        /// - `amount_out`: Amount asset will be bought
        /// - `amount_in_max`: Maximum amount of sold asset
        /// - `path`: path can convert to pairs.
        /// - `to`: Account that receive the target asset
        /// - `deadline`: Height of the cutoff block of this transaction
        #[weight = 0]
        #[transactional]
        pub fn swap_tokens_for_exact_tokens(
            origin,
            #[compact] amount_out: TokenBalance,
            #[compact] amount_in_max: TokenBalance,
            path: Vec<AssetId>,
            to: <T::Lookup as StaticLookup>::Source,
            #[compact] deadline: T::BlockNumber,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let to = T::Lookup::lookup(to)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            Self::inner_swap_tokens_for_exact_tokens(&who, amount_out, amount_in_max, &path, &to)?;

            Self::deposit_event(RawEvent::TokenSwap(who, to, path));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    pub(crate) fn is_reachable(para_id: ParaId) -> bool {
        T::TargetChains::get()
            .iter()
            .map(|(location, _)| location)
            .any(|l| *l == make_x2_location(para_id.into()))
    }

    // Check the native currency must be more than ExistentialDeposit,
    // other assets always return true
    pub(crate) fn must_more_than_minimum(
        asset_id: AssetId,
        para_id: ParaId,
        amount: TokenBalance,
    ) -> bool {
        if asset_id.module_index != NATIVE_CURRENCY {
            return true;
        }

        T::TargetChains::get()
            .iter()
            .find(|(l, _)| *l == make_x2_location(para_id.into()))
            .map(|&(_, minimum_balance)| amount >= minimum_balance)
            .unwrap_or(false)
    }

    // Make the deposit asset order
    fn make_deposit_asset_order(account: T::AccountId) -> Order {
        Order::DepositAsset {
            assets: vec![MultiAsset::All],
            dest: MultiLocation::X1(Junction::AccountId32 {
                network: NetworkId::Any,
                id: T::AccountId32Converter::convert(account),
            }),
        }
    }

    // Transfer zenlink assets which are native to this parachain
    pub(crate) fn make_xcm_lateral_transfer_native(
        location: MultiLocation,
        para_id: ParaId,
        account: T::AccountId,
        amount: TokenBalance,
    ) -> Xcm {
        Xcm::WithdrawAsset {
            assets: vec![MultiAsset::ConcreteFungible { id: location, amount }],
            effects: vec![Order::DepositReserveAsset {
                assets: vec![MultiAsset::All],
                dest: make_x2_location(para_id.into()),
                effects: vec![Self::make_deposit_asset_order(account)],
            }],
        }
    }
    // Transfer zenlink assets which are foreign to this parachain
    pub(crate) fn make_xcm_lateral_transfer_foreign(
        reserve_chain: ParaId,
        location: MultiLocation,
        para_id: ParaId,
        account: T::AccountId,
        amount: TokenBalance,
    ) -> Xcm {
        Xcm::WithdrawAsset {
            assets: vec![MultiAsset::ConcreteFungible { id: location, amount }],
            effects: vec![Order::InitiateReserveWithdraw {
                assets: vec![MultiAsset::All],
                reserve: make_x2_location(reserve_chain.into()),
                effects: vec![if para_id == reserve_chain {
                    Self::make_deposit_asset_order(account)
                } else {
                    Order::DepositReserveAsset {
                        assets: vec![MultiAsset::All],
                        dest: make_x2_location(para_id.into()),
                        effects: vec![Self::make_deposit_asset_order(account)],
                    }
                }],
            }],
        }
    }

    pub(crate) fn make_xcm_transfer_to_parachain(
        asset_id: &AssetId,
        para_id: ParaId,
        account: &T::AccountId,
        amount: TokenBalance,
    ) -> Result<Xcm, XcmError> {
        let asset_location = MultiLocation::X4(
            Junction::Parent,
            Junction::Parachain { id: asset_id.chain_id },
            Junction::PalletInstance { id: asset_id.module_index },
            Junction::GeneralIndex { id: asset_id.asset_index as u128 },
        );

        let check_foreign = |id| {
            if Self::a_is_manageable(id) {
                match Self::asset_property(id) {
                    AssetProperty::Foreign => return Ok(true),
                    AssetProperty::Lp(_) => return Ok(false),
                }
            }

            if T::NativeCurrency::a_is_manageable(id) {
                return Ok(false);
            }

            if T::OtherAssets::a_is_manageable(id) {
                return Ok(false);
            }

            Err(XcmError::FailedToTransactAsset("Invalid AssetId"))
        };

        check_foreign(*asset_id).map(|is_foreign| {
            if is_foreign {
                Ok(Self::make_xcm_lateral_transfer_foreign(
                    ParaId::from(asset_id.chain_id),
                    asset_location,
                    para_id,
                    account.clone(),
                    amount,
                ))
            } else {
                Ok(Self::make_xcm_lateral_transfer_native(
                    asset_location,
                    para_id,
                    account.clone(),
                    amount,
                ))
            }
        })?
    }
}
