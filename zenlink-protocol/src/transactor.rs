// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.
#![allow(unused_variables)]

use super::*;
use xcm::latest::{multiasset, Fungibility};

/// Asset transaction errors.
enum Error {
	/// `MultiLocation` to `AccountId` Conversion failed.
	AccountIdConversionFailed,
}

impl From<Error> for XcmError {
	fn from(e: Error) -> Self {
		match e {
			Error::AccountIdConversionFailed => XcmError::FailedToTransactAsset("AccountIdConversionFailed"),
		}
	}
}

pub struct TrustedParas<ParaChains>(PhantomData<ParaChains>);

impl<ParaChains: Get<Vec<(MultiLocation, u128)>>> FilterAssetLocation for TrustedParas<ParaChains> {
	fn filter_asset_location(_asset: &MultiAsset, origin: &MultiLocation) -> bool {
		log::info!(target: LOG_TARGET, "filter_asset_location: origin = {:?}", origin);

		ParaChains::get()
			.iter()
			.map(|(location, _)| location)
			.any(|l| *l == *origin)
	}
}

pub struct TransactorAdaptor<ZenlinkAssets, AccountIdConverter, AccountId>(
	PhantomData<(ZenlinkAssets, AccountIdConverter, AccountId)>,
);

impl<
		ZenlinkAssets: MultiAssetsHandler<AccountId>,
		AccountIdConverter: Convert<MultiLocation, AccountId>,
		AccountId: sp_std::fmt::Debug + Clone,
	> TransactAsset for TransactorAdaptor<ZenlinkAssets, AccountIdConverter, AccountId>
{
	fn deposit_asset(asset: &MultiAsset, who: &MultiLocation) -> XcmResult {
		log::info!(
			target: LOG_TARGET,
			"deposit_asset: asset = {:?}, who = {:?}",
			asset,
			who,
		);

		let who = AccountIdConverter::convert_ref(who).map_err(|()| Error::AccountIdConversionFailed)?;

		match &asset.id {
			multiasset::AssetId::Concrete(location) => {
				if let Fungibility::Fungible(amount) = asset.fun {
					let asset_id =
						multilocation_to_asset(location).ok_or(XcmError::FailedToTransactAsset("unKnown asset"))?;

					ZenlinkAssets::deposit(asset_id, &who, amount)
						.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
				} else {
					return Err(XcmError::AssetNotFound);
				}
				Ok(())
			}
			_ => Err(XcmError::AssetNotFound),
		}
	}

	fn withdraw_asset(asset: &MultiAsset, who: &MultiLocation) -> Result<Assets, XcmError> {
		log::info!(
			target: LOG_TARGET,
			"withdraw_asset: asset = {:?}, who = {:?}",
			asset,
			who,
		);

		let who = AccountIdConverter::convert_ref(who).map_err(|()| Error::AccountIdConversionFailed)?;

		match &asset.id {
			multiasset::AssetId::Concrete(location) => {
				if let Fungibility::Fungible(amount) = asset.fun {
					let asset_id =
						multilocation_to_asset(location).ok_or(XcmError::FailedToTransactAsset("unKnown asset"))?;

					ZenlinkAssets::withdraw(asset_id, &who, amount)
						.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
					Ok(asset.clone().into())
				} else {
					Err(XcmError::NotWithdrawable)
				}
			}
			_ => Err(XcmError::NotWithdrawable),
		}
	}
}

fn multilocation_to_asset(location: &MultiLocation) -> Option<AssetId> {
	if location.parents != 1 {
		return None;
	}

	match location.interior {
		Junctions::X3(
			Junction::Parachain(chain_id),
			Junction::PalletInstance(asset_type),
			Junction::GeneralIndex(asset_index),
		) => Some(AssetId {
			chain_id,
			asset_type,
			asset_index: asset_index as u64,
		}),
		_ => None,
	}
}
