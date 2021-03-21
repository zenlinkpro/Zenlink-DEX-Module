// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.
#![allow(unused_variables)]

use frame_support::traits::Get;
use sp_std::{marker::PhantomData, prelude::Vec};

use crate::{
    AssetId, FilterAssetLocation, Junction, LocationConversion, MultiAsset, MultiLocation, ParaId,
    TokenBalance, TransactAsset, XcmError, XcmResult, ZenlinkMultiAsset, LOG_TARGET,
};

/// Asset transaction errors.
enum Error {
    /// `MultiLocation` to `AccountId` Conversion failed.
    AccountIdConversionFailed,
    /// Zenlink only use X4 format xcm
    XcmNotX4Format,
    /// Zenlink only use MultiAsset::ConcreteFungible
    XcmNotConcreteFungible,
}

impl From<Error> for XcmError {
    fn from(e: Error) -> Self {
        match e {
            Error::AccountIdConversionFailed => {
                XcmError::FailedToTransactAsset("AccountIdConversionFailed")
            }
            Error::XcmNotX4Format => XcmError::FailedToTransactAsset("XcmNotX4Format"),
            Error::XcmNotConcreteFungible => {
                XcmError::FailedToTransactAsset("XcmNotConcreteFungible")
            }
        }
    }
}

pub struct ParaChainWhiteList<ParachainList>(PhantomData<ParachainList>);

impl<ParachainList: Get<Vec<MultiLocation>>> FilterAssetLocation
    for ParaChainWhiteList<ParachainList>
{
    fn filter_asset_location(_asset: &MultiAsset, origin: &MultiLocation) -> bool {
        log::info!(target: LOG_TARGET, "filter_asset_location: origin = {:?}", origin);

        ParachainList::get().contains(origin)
    }
}

pub struct Transactor<ZenlinkAssets, AccountIdConverter, AccountId, ParaChainId>(
    PhantomData<(ZenlinkAssets, AccountIdConverter, AccountId, ParaChainId)>,
);

impl<
        ZenlinkAssets: ZenlinkMultiAsset<AccountId, TokenBalance>,
        AccountIdConverter: LocationConversion<AccountId>,
        AccountId: sp_std::fmt::Debug,
        ParaChainId: Get<ParaId>,
    > TransactAsset for Transactor<ZenlinkAssets, AccountIdConverter, AccountId, ParaChainId>
{
    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
        log::info!(
            target: LOG_TARGET,
            "deposit_asset: asset = {:?}, location = {:?}",
            asset,
            location,
        );

        let who = AccountIdConverter::from_location(location)
            .ok_or_else(|| XcmError::from(Error::AccountIdConversionFailed))?;

        match asset {
            MultiAsset::ConcreteFungible { id, amount } => {
                if let Some(asset_id) = multilocation_to_asset(id) {
                    ZenlinkAssets::multi_asset_deposit(&asset_id, &who, *amount)
                        .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

                    Ok(())
                } else {
                    Err(XcmError::from(Error::XcmNotX4Format))
                }
            }
            _ => Err(XcmError::from(Error::XcmNotConcreteFungible)),
        }
    }

    fn withdraw_asset(
        asset: &MultiAsset,
        location: &MultiLocation,
    ) -> Result<MultiAsset, XcmError> {
        log::info!(
            target: LOG_TARGET,
            "withdraw_asset: asset = {:?}, location = {:?}",
            asset,
            location,
        );

        let who = AccountIdConverter::from_location(location)
            .ok_or_else(|| XcmError::from(Error::AccountIdConversionFailed))?;

        match asset {
            MultiAsset::ConcreteFungible { id, amount } => {
                if let Some(asset_id) = multilocation_to_asset(id) {
                    ZenlinkAssets::multi_asset_withdraw(&asset_id, &who, *amount)
                        .map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

                    Ok(asset.clone())
                } else {
                    Err(XcmError::from(Error::XcmNotX4Format))
                }
            }
            _ => Err(XcmError::from(Error::XcmNotConcreteFungible)),
        }
    }
}

fn multilocation_to_asset(location: &MultiLocation) -> Option<AssetId> {
    match location {
        MultiLocation::X4(
            Junction::Parent,
            Junction::Parachain { id: chain_id },
            Junction::PalletInstance { id: pallet_index },
            Junction::GeneralIndex { id: asset_index },
        ) => Some(AssetId {
            chain_id: *chain_id,
            module_index: *pallet_index,
            asset_index: (*asset_index) as u32,
        }),
        _ => None,
    }
}
