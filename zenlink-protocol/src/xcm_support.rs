// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.
#![allow(unused_variables)]
use crate::{
	AssetId, FilterAssetLocation, Junction, LocationConversion, MultiAsset, MultiLocation, ParaId,
	TokenBalance, TransactAsset, XcmError, XcmResult, ZenlinkMultiAsset,
};
use frame_support::traits::{Currency, ExistenceRequirement, Get, WithdrawReasons};
use sp_std::{convert::TryFrom, marker::PhantomData, prelude::Vec};

pub struct ParaChainWhiteList<ParachainList>(PhantomData<ParachainList>);

impl<ParachainList: Get<Vec<MultiLocation>>> FilterAssetLocation
	for ParaChainWhiteList<ParachainList>
{
	fn filter_asset_location(_asset: &MultiAsset, origin: &MultiLocation) -> bool {
		frame_support::debug::print!("filter_asset_location {:?}", origin);
		sp_std::if_std! {println!("zenlink::<filter_asset_location> {:?}", origin)}

		ParachainList::get().contains(origin)
	}
}

pub struct Transactor<NativeCurrency, ZenlinkAssets, AccountIdConverter, AccountId, ParaChainId>(
	PhantomData<(
		NativeCurrency,
		ZenlinkAssets,
		AccountIdConverter,
		AccountId,
		ParaChainId,
	)>,
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
		sp_std::if_std! { println!("zenlink::<deposit_asset> who = {:?}", who); }

		match asset {
			MultiAsset::ConcreteFungible { id, amount } => {
				sp_std::if_std! { println!("zenlink::<deposit_asset> ConcreteFungible: id = {:?}, amount = {:?}", id, amount); }
				Self::deposit(id, &who, *amount)
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
		sp_std::if_std! { println!("zenlink::<withdraw_asset> who = {:?}", who); }

		match asset {
			MultiAsset::ConcreteFungible { id, amount } => {
				sp_std::if_std! { println!("zenlink::<withdraw_asset> ConcreteFungible id = {:?}, amount = {:?}", id, amount); }
				Self::withdraw(id, &who, *amount).map(|_| asset.clone())
			}
			_ => {
				sp_std::if_std! { println!("zenlink::<withdraw_asset> Undefined asset = {:?}", asset); }
				Err(XcmError::Undefined)
			}
		}
	}
}

impl<
		NativeCurrency: Currency<AccountId>,
		ZenlinkAssets: ZenlinkMultiAsset<AccountId, TokenBalance>,
		AccountIdConverter: LocationConversion<AccountId>,
		AccountId: sp_std::fmt::Debug,
		ParaChainId: Get<ParaId>,
	> Transactor<NativeCurrency, ZenlinkAssets, AccountIdConverter, AccountId, ParaChainId>
{
	fn deposit(id: &MultiLocation, who: &AccountId, amount: u128) -> XcmResult {
		let length = id.len();
		match id {
            // Deposit Zenlink Assets(ParaCurrency)
            MultiLocation::X2(
                Junction::PalletInstance { .. },
                Junction::GeneralIndex { id }) => {
				deposit_para_currency::<
					AccountId,
					ParaChainId,
					ZenlinkAssets
				>(who, amount, id)
			}
            // Deposit native Currency
            // length == 2
            MultiLocation::X2(
                Junction::Parent,
                Junction::Parachain { id })
            // Deposit native Currency
            // From a parachain which is not the reserve parachain
            // length == 4
            | MultiLocation::X4(
                Junction::Parent,
                Junction::Parachain { id },
                Junction::PalletInstance { .. },
                Junction::GeneralIndex { .. })
            => {
				deposit_native_currency::<
					AccountId,
					ParaChainId,
					NativeCurrency
				>(who, amount, length, id)
			}
            _ => {
                sp_std::if_std! { println!("zenlink::<deposit> Undefined id = {:?}", id); }
                Err(XcmError::Undefined)
            }
        }
	}

	fn withdraw(id: &MultiLocation, who: &AccountId, amount: u128) -> XcmResult {
		let length = id.len();
		match id {
            // Withdraw Zenlink Assets(ParaCurrency)
            MultiLocation::X2(
                Junction::PalletInstance { .. },
                Junction::GeneralIndex { id },
            ) => {
				withdraw_para_currency::<
					AccountId,
					ParaChainId,
					ZenlinkAssets
				>(who, amount, id)
			},
            // Withdraw native Currency
            // length == 2
            MultiLocation::X2(
                Junction::Parent,
                Junction::Parachain { id }
            )
            // Withdraw native Currency
            // From a parachain which is not the reserve parachain
            // length == 4
            | MultiLocation::X4(
                Junction::Parent,
                Junction::Parachain { id },
                Junction::PalletInstance { .. },
                Junction::GeneralIndex { .. },
            ) => {
				withdraw_native_currency::<
					AccountId,
					ParaChainId,
					NativeCurrency
				>(who, amount, length, id)
			}
            _ => {
                sp_std::if_std! { println!("zenlink::<withdraw> Undefined id = {:?}", id); }
                Err(XcmError::Undefined)
            }
        }
	}
}

fn withdraw_para_currency<AccountId, ParaChainId, ZenlinkAssets>(
	who: &AccountId,
	amount: u128,
	id: &u128,
) -> Result<(), XcmError>
where
	AccountId: sp_std::fmt::Debug,
	ParaChainId: Get<ParaId>,
	ZenlinkAssets: ZenlinkMultiAsset<AccountId, TokenBalance>,
{
	sp_std::if_std! { println!("zenlink::<withdraw> amount = {:?}", amount); }
	let asset_id = AssetId::from(*id);
	let para_id: u32 = ParaChainId::get().into();
	if para_id as u128 == *id {
		sp_std::if_std! { println!("zenlink::<withdraw> para_id = {:?}, id = {:?}", para_id, id); }
		return Err(XcmError::Unimplemented);
	}
	// asset_id must be ParaCurrency
	ZenlinkAssets::withdraw(asset_id, &who, amount as TokenBalance).map_err(|err| {
		sp_std::if_std! { println!("zenlink::<withdraw> err = {:?}", err); }
		XcmError::Undefined
	})?;
	sp_std::if_std! { println!("zenlink::<withdraw> success"); }

	Ok(())
}

fn withdraw_native_currency<AccountId, ParaChainId, NativeCurrency>(
	who: &AccountId,
	amount: u128,
	length: usize,
	id: &u32,
) -> Result<(), XcmError>
where
	AccountId: sp_std::fmt::Debug,
	ParaChainId: Get<ParaId>,
	NativeCurrency: Currency<AccountId>,
{
	let para_id: u32 = ParaChainId::get().into();
	sp_std::if_std! { println!("zenlink::<withdraw> amount = {:?}, para_id = {:?}", amount, para_id); }

	if (length == 2 && *id == para_id) || (length == 4 && *id != para_id) {
		let value =
			<<NativeCurrency as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(amount)
				.map_err(|_| {
				sp_std::if_std! { println!("zenlink::<withdraw> amount convert to Balance failed"); }
			})?;
		let _ = NativeCurrency::withdraw(
			&who,
			value,
			WithdrawReasons::TRANSFER,
			ExistenceRequirement::AllowDeath,
		)
		.map_err(|_| XcmError::Undefined)?;
		sp_std::if_std! { println!("zenlink::<withdraw> success"); }
		Ok(())
	} else {
		sp_std::if_std! { println!("zenlink::<withdraw> discard"); }
		Err(XcmError::UnhandledXcmMessage)
	}
}

fn deposit_para_currency<AccountId, ParaChainId, ZenlinkAssets>(
	who: &AccountId,
	amount: u128,
	id: &u128,
) -> Result<(), XcmError>
where
	AccountId: sp_std::fmt::Debug,
	ParaChainId: Get<ParaId>,
	ZenlinkAssets: ZenlinkMultiAsset<AccountId, TokenBalance>,
{
	sp_std::if_std! { println!("zenlink::<deposit> amount = {:?}", amount); }
	let asset_id = AssetId::from(*id);
	let para_id: u32 = ParaChainId::get().into();
	if para_id as u128 == *id {
		sp_std::if_std! { println!("zenlink::<deposit> para_id = {:?}, id = {:?}", para_id, id); }
		return Err(XcmError::Unimplemented);
	}
	//  asset_id must be ParaCurrency
	ZenlinkAssets::deposit(asset_id, &who, amount as TokenBalance).map_err(|err| {
		sp_std::if_std! { println!("zenlink::<deposit> err = {:?}", err); }
		XcmError::Undefined
	})?;
	sp_std::if_std! { println!("zenlink::<deposit> success"); }

	Ok(())
}

fn deposit_native_currency<AccountId, ParaChainId, NativeCurrency>(
	who: &AccountId,
	amount: u128,
	length: usize,
	id: &u32,
) -> Result<(), XcmError>
where
	AccountId: sp_std::fmt::Debug,
	ParaChainId: Get<ParaId>,
	NativeCurrency: Currency<AccountId>,
{
	let para_id: u32 = ParaChainId::get().into();
	sp_std::if_std! { println!("zenlink::<deposit> amount = {:?}, para_id = {:?}", amount, para_id); }

	if (length == 2 && *id == para_id) || (length == 4 && *id != para_id) {
		let value =
			<<NativeCurrency as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(amount)
				.map_err(|_| {
				sp_std::if_std! { println!("zenlink::<deposit> amount convert to Balance failed"); }
			})?;
		let _ = NativeCurrency::deposit_creating(&who, value);
		sp_std::if_std! { println!("zenlink::<deposit> success"); }
		Ok(())
	} else {
		sp_std::if_std! { println!("zenlink::<deposit> discard"); }
		Err(XcmError::UnhandledXcmMessage)
	}
}
