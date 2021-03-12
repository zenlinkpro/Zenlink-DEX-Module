// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.
#![allow(unused_variables)]

use frame_support::traits::{Currency, Get};
use sp_std::{marker::PhantomData, prelude::Vec};

use crate::{
    AssetId, FilterAssetLocation, Junction, LocationConversion, MultiAsset, MultiLocation, ParaId,
    TokenBalance, TransactAsset, XcmError, XcmResult, ZenlinkMultiAsset,
};

pub struct ParaChainWhiteList<ParachainList>(PhantomData<ParachainList>);

impl<ParachainList: Get<Vec<MultiLocation>>> FilterAssetLocation
    for ParaChainWhiteList<ParachainList>
{
    fn filter_asset_location(_asset: &MultiAsset, origin: &MultiLocation) -> bool {
        log::debug!("filter_asset_location {:?}", origin);
        sp_std::if_std! {println!("zenlink::<filter_asset_location> {:?}", origin)}

        ParachainList::get().contains(origin)
    }
}

pub struct Transactor<NativeCurrency, ZenlinkAssets, AccountIdConverter, AccountId, ParaChainId>(
    PhantomData<(NativeCurrency, ZenlinkAssets, AccountIdConverter, AccountId, ParaChainId)>,
);

impl<
        NativeCurrency: Currency<AccountId>,
        ZenlinkAssets: ZenlinkMultiAsset<AccountId, TokenBalance>,
        AccountIdConverter: LocationConversion<AccountId>,
        AccountId: sp_std::fmt::Debug,
        ParaChainId: Get<ParaId>,
    > TransactAsset
    for Transactor<NativeCurrency, ZenlinkAssets, AccountIdConverter, AccountId, ParaChainId>
{
    fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
        sp_std::if_std! { println!("zenlink::<deposit_asset> asset = {:?}, location = {:?}", asset, location); }
        let who = AccountIdConverter::from_location(location).ok_or(())?;

        match asset {
            MultiAsset::ConcreteFungible { id, amount } => {
                if let Some(asset_id) = multilocation_to_asset(id) {
                    ZenlinkAssets::multi_asset_deposit(&asset_id, &who, *amount)
                        .or(Err(XcmError::Undefined))
                } else {
                    Err(XcmError::Undefined)
                }
            }
            _ => {
                sp_std::if_std! { println!("zenlink::<deposit_asset> Undefined: asset = {:?}", asset); }
                Err(XcmError::Undefined)
            }
        }
    }

    fn withdraw_asset(
        asset: &MultiAsset,
        location: &MultiLocation,
    ) -> Result<MultiAsset, XcmError> {
        sp_std::if_std! { println!("zenlink::<withdraw_asset> asset = {:?}, location = {:?}", asset, location); }
        let who = AccountIdConverter::from_location(location).ok_or(())?;
        match asset {
            MultiAsset::ConcreteFungible { id, amount } => {
                if let Some(asset_id) = multilocation_to_asset(id) {
                    ZenlinkAssets::multi_asset_withdraw(&asset_id, &who, *amount)
                        .map_or(Err(XcmError::EscalationOfPrivilege), |_| Ok(asset.clone()))
                } else {
                    Err(XcmError::Undefined)
                }
            }
            _ => {
                sp_std::if_std! { println!("zenlink::<deposit> Undefined asset = {:?}", asset); }
                Err(XcmError::Undefined)
            }
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
        ) => {
            sp_std::if_std! { println!("zenlink::<multilocation_to_asset> chain_id = {:?}, pallet_index = {:#?}, asset_index = {:#?}",
            chain_id, pallet_index, asset_index); }

            Some(AssetId {
                chain_id: *chain_id,
                module_index: *pallet_index,
                asset_index: (*asset_index) as u32,
            })
        }
        _ => {
            sp_std::if_std! { println!("zenlink::<multilocation_to_asset> None")}
            None
        }
    }
}
