//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.
#![allow(unused_variables)]
use crate::{
	primitives::{CrossChainOperation, OperateAsset},
	AssetId, FilterAssetLocation, Junction, LocationConversion, MultiAsset, MultiLocation, ParaId,
	TokenBalance, TransactAsset, XcmError, XcmResult, ZenlinkMultiAsset,
};
use codec::Decode;
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
		ZenlinkAssets: ZenlinkMultiAsset<AccountId, TokenBalance> + OperateAsset<AccountId, TokenBalance>,
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
			MultiAsset::AbstractFungible { id, amount } => {
				sp_std::if_std! { println!("zenlink::<deposit_asset> AbstractFungible: amount = {:?}", amount); }
				Self::deposit_abstract_asset(id, &who, *amount)
			}
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
			MultiAsset::AbstractFungible { id, amount } => {
				sp_std::if_std! { println!("zenlink::<withdraw_asset> AbstractFungible amount = {:?}", amount); }
				Self::withdraw_abstract_asset(id, &who, *amount).map(|_| asset.clone())
			}
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
		ZenlinkAssets: ZenlinkMultiAsset<AccountId, TokenBalance> + OperateAsset<AccountId, TokenBalance>,
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
                sp_std::if_std! { println!("zenlink::<deposit> amount = {:?}", amount); }
                let asset_id = AssetId::from(*id);
                let para_id: u32 = ParaChainId::get().into();
                if para_id as u128 == *id {
                    sp_std::if_std! { println!("zenlink::<deposit> para_id = {:?}, id = {:?}", para_id, id); }
                    return Err(XcmError::Unimplemented);
                }
                //  asset_id must be ParaCurrency
                ZenlinkAssets::deposit(asset_id, &who, amount as TokenBalance)
                    .map_err(|err| {
                        sp_std::if_std! { println!("zenlink::<deposit> err = {:?}", err); }
                        XcmError::Undefined
                    })?;
                sp_std::if_std! { println!("zenlink::<deposit> success"); }
                Ok(())
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
                let para_id: u32 = ParaChainId::get().into();
                sp_std::if_std! { println!("zenlink::<deposit> amount = {:?}, para_id = {:?}", amount, para_id); }

                if (length == 2 && *id == para_id)
                    || (length == 4 && *id != para_id) {
                    let value = <<NativeCurrency as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(amount)
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
                let para_id: u32 = ParaChainId::get().into();
                sp_std::if_std! { println!("zenlink::<withdraw> amount = {:?}, para_id = {:?}", amount, para_id); }

                if (length == 2 && *id == para_id)
                    || (length == 4 && *id != para_id) {
                    let value = <<NativeCurrency as Currency<AccountId>>::Balance as TryFrom<u128, >>::try_from(amount)
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
            _ => {
                sp_std::if_std! { println!("zenlink::<withdraw> Undefined id = {:?}", id); }
                Err(XcmError::Undefined)
            }
        }
	}

	fn deposit_abstract_asset(id: &[u8], who: &AccountId, amount: u128) -> XcmResult {
		let operate = CrossChainOperation::decode(&mut &id[..]).unwrap();
		let para_id: u32 = ParaChainId::get().into();
		match operate {
			CrossChainOperation::AddLiquidity {
				origin_chain,
				target_chain,
				token_0,
				token_1,
				amount_1,
			} => {
				sp_std::if_std! {
					println!("zenlink::<AddLiquidity> \
					origin = {:?}, target = {:?}, token_0 = {:?}, token_1 = {:?}, amount_1 = {:?}",
					origin_chain, target_chain, token_0, token_1, amount_1);
				}

				match para_id {
					// Local execution
					x if x == origin_chain => {
						let value = <<NativeCurrency as Currency<AccountId>>::Balance as TryFrom<
							u128,
						>>::try_from(amount)
						.map_err(|_| {
							sp_std::if_std! { println!("zenlink::<AddLiquidity> amount convert to Balance failed"); }
							XcmError::Undefined
						})?;
						let _ = NativeCurrency::deposit_creating(&who, value);

						Ok(())
					}
					// Remote execution
					x if x == target_chain => {
						ZenlinkAssets::deposit(
							AssetId::from(token_0),
							&who,
							amount as TokenBalance,
						)
						.map_err(|err| {
							sp_std::if_std! { println!("zenlink::<AddLiquidity> err = {:?}", err); }
							XcmError::Undefined
						})?;

						let despoit_amount =
							ZenlinkAssets::balance_of(AssetId::from(token_0), &who);

						sp_std::if_std! { println!("zenlink::<AddLiquidity> deposit amount = {:?}", despoit_amount); }

						ZenlinkAssets::add_liquidity(
							&who,
							&AssetId::from(token_0),
							&AssetId::NativeCurrency,
							despoit_amount,
							amount_1,
						)
						.map_err(|err| {
							sp_std::if_std! { println!("zenlink::<AddLiquidity> err = {:?}", err); }
							XcmError::Undefined
						})?;

						Ok(())
					}
					_ => {
						sp_std::if_std! { println!("zenlink::<AddLiquidity> Undefined para_id = {:?}", para_id); }

						Err(XcmError::Undefined)
					}
				}
			}
			CrossChainOperation::SwapExactTokensForTokens {
				origin_chain,
				target_chain,
				amount_out_min,
				path,
			} => {
				sp_std::if_std! {
					println!("zenlink::<SwapExactTokensForTokens> \
					origin = {:?}, target = {:?}, amount = {:?}, path = {:#?}",
					origin_chain, target_chain, amount_out_min, path);
				}

				match para_id {
					// Local execution
					x if x == origin_chain => {
						let value = <<NativeCurrency as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(amount)
                            .map_err(|_| {
                                sp_std::if_std! { println!("zenlink::<SwapExactTokensForTokens> amount convert to Balance failed"); }
                                XcmError::Undefined
                            })?;
						let _ = NativeCurrency::deposit_creating(&who, value);

						Ok(())
					}
					// Remote execution
					x if x == target_chain => {
						let asset_id_path: Vec<AssetId> = path
							.iter()
							.map(|id| {
								if *id == para_id {
									AssetId::NativeCurrency
								} else {
									AssetId::from(*id)
								}
							})
							.collect();

						ZenlinkAssets::deposit(asset_id_path[0], &who, amount as TokenBalance)
							.map_err(|err| {
								sp_std::if_std! { println!("zenlink::<SwapExactTokensForTokens> err = {:?}", err); }
								XcmError::Undefined
							})?;

						let despoit_amount = ZenlinkAssets::balance_of(asset_id_path[0], &who);

						sp_std::if_std! { println!("zenlink::<SwapExactTokensForTokens> deposit amount = {:?}", despoit_amount); }

						ZenlinkAssets::swap_in(
							&who,
							despoit_amount,
							amount_out_min,
							&asset_id_path,
						)
						.map_err(|err| {
							sp_std::if_std! { println!("zenlink::<SwapExactTokensForTokens> err = {:?}", err); }
							XcmError::Undefined
						})?;

						Ok(())
					}
					_ => {
						sp_std::if_std! { println!("zenlink::<SwapExactTokensForTokens> Undefined para_id = {:?}", para_id); }

						Err(XcmError::Undefined)
					}
				}
			}
			CrossChainOperation::SwapTokensForExactTokens {
				origin_chain,
				target_chain,
				amount_out,
				path,
			} => {
				sp_std::if_std! {
					println!("zenlink::<SwapTokensForExactTokens> \
					origin = {:?}, target = {:?}, amount = {:?}, path = {:#?}",
					origin_chain, target_chain, amount_out, path);
				}

				match para_id {
					// Local execution
					x if x == origin_chain => {
						let value = <<NativeCurrency as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(amount)
                            .map_err(|_| {
                                sp_std::if_std! { println!("zenlink::<SwapTokensForExactTokens> amount convert to Balance failed"); }
                                XcmError::Undefined
                            })?;
						let _ = NativeCurrency::deposit_creating(&who, value);

						Ok(())
					}
					// Remote execution
					x if x == target_chain => {
						let asset_id_path: Vec<AssetId> = path
							.iter()
							.map(|id| {
								if *id == para_id {
									AssetId::NativeCurrency
								} else {
									AssetId::from(*id)
								}
							})
							.collect();

						ZenlinkAssets::deposit(asset_id_path[0], &who, amount as TokenBalance)
							.map_err(|err| {
								sp_std::if_std! { println!("zenlink::<SwapTokensForExactTokens> err = {:?}", err); }
								XcmError::Undefined
							})?;

						let despoit_amount = ZenlinkAssets::balance_of(asset_id_path[0], &who);

						sp_std::if_std! { println!("zenlink::<SwapTokensForExactTokens> deposit amount = {:?}", despoit_amount); }

						ZenlinkAssets::swap_out(
							&who,
							amount_out,
							amount as TokenBalance,
							&asset_id_path,
						)
						.map_err(|err| {
							sp_std::if_std! { println!("zenlink::<SwapTokensForExactTokens> err = {:?}", err); }
							XcmError::Undefined
						})?;

						Ok(())
					}
					_ => {
						sp_std::if_std! { println!("zenlink::<SwapTokensForExactTokens> Undefined para_id = {:?}", para_id); }

						Err(XcmError::Undefined)
					}
				}
			}
		}
	}

	fn withdraw_abstract_asset(
		id: &[u8],
		who: &AccountId,
		amount: u128,
	) -> Result<MultiAsset, XcmError> {
		sp_std::if_std! { println!("zenlink::<withdraw_abstract_asset> amount = {:?}", amount); }
		let value =
			<<NativeCurrency as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(amount)
				.map_err(|_| {
				sp_std::if_std! { println!("zenlink::<withdraw_abstract_asset> amount convert to Balance failed"); }
			})?;

		let _ = NativeCurrency::withdraw(
			&who,
			value,
			WithdrawReasons::TRANSFER,
			ExistenceRequirement::AllowDeath,
		)
		.map_err(|_| XcmError::Undefined)?;

		sp_std::if_std! { println!("zenlink::<withdraw_abstract_asset> success"); }

		Ok(MultiAsset::AbstractFungible {
			id: id.to_vec(),
			amount,
		})
	}
}
