// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use codec::{Decode, Encode};
use sp_runtime::traits::Hash;
use sp_std::{
    convert::{TryFrom, TryInto},
    vec,
};

use crate::{
    sp_api_hidden_includes_decl_storage::hidden_include::traits::PalletInfo,
    AssetId, Config, Convert, DownwardMessageHandler, ExecuteXcm, Get, HrmpMessageHandler,
    HrmpMessageSender, InboundDownwardMessage, InboundHrmpMessage, Junction, Module, MultiAsset,
    MultiLocation, NetworkId, Order, OutboundHrmpMessage, ParaId,
    RawEvent::{XcmBadFormat, XcmBadVersion, XcmExecuteFail, HrmpMessageSent, XcmExecuteSuccess, UpwardMessageSent},
    SendXcm, TokenBalance, UpwardMessageSender, Vec, VersionedXcm, Xcm, XcmError,
};

/// Origin for the parachains module.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Origin {
    /// It comes from the (parent) relay chain.
    Relay,
    /// It comes from a (sibling) parachain.
    SiblingParachain(ParaId),
}

impl From<ParaId> for Origin {
    fn from(id: ParaId) -> Origin {
        Origin::SiblingParachain(id)
    }
}

impl From<u32> for Origin {
    fn from(id: u32) -> Origin {
        Origin::SiblingParachain(id.into())
    }
}

impl<T: Config> Module<T> {
    // Return Zenlink Protocol Pallet index
    fn index() -> u8 {
        T::PalletInfo::index::<Self>().map_or(0u8, |index| index as u8)
    }

    // Return true if the parachain id is in the set of the target parachains
    pub(crate) fn is_reachable(para_id: ParaId) -> bool {
        let location = MultiLocation::X2(Junction::Parent, Junction::Parachain { id: para_id.into() });
        T::TargetChains::get().contains(&location)
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
                dest: MultiLocation::X2(
                    Junction::Parent,
                    Junction::Parachain { id: para_id.into() },
                ),
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
                reserve: MultiLocation::X2(
                    Junction::Parent,
                    Junction::Parachain { id: reserve_chain.into() },
                ),
                effects: vec![if para_id == reserve_chain {
                    Self::make_deposit_asset_order(account)
                } else {
                    Order::DepositReserveAsset {
                        assets: vec![MultiAsset::All],
                        dest: MultiLocation::X2(
                            Junction::Parent,
                            Junction::Parachain { id: para_id.into() },
                        ),
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
    ) -> Xcm {
        match asset_id {
            AssetId::NativeCurrency => {
                let location = MultiLocation::X2(
                    Junction::Parent,
                    Junction::Parachain { id: T::ParaId::get().into() },
                );

                Self::make_xcm_lateral_transfer_native(location, para_id, account.clone(), amount)
            }
            AssetId::ParaCurrency(id) => {
                let location = MultiLocation::X2(
                    Junction::PalletInstance { id: Self::index() },
                    Junction::GeneralIndex { id: (*id) as u128 },
                );

                Self::make_xcm_lateral_transfer_foreign(
                    (*id).into(),
                    location,
                    para_id,
                    account.clone(),
                    amount,
                )
            }
        }
    }
}

impl<T: Config> DownwardMessageHandler for Module<T> {
    fn handle_downward_message(msg: InboundDownwardMessage) {
        let hash = msg.using_encoded(T::Hashing::hash);
        frame_support::debug::print!("Processing Downward XCM: hash = {:?}", &hash);
        match VersionedXcm::decode(&mut &msg.msg[..]).map(Xcm::try_from) {
            Ok(Ok(xcm)) => {
                match T::XcmExecutor::execute_xcm(Junction::Parent.into(), xcm.clone()) {
                    Ok(..) => Self::deposit_event(XcmExecuteSuccess(hash)),
                    Err(_e @ XcmError::UnhandledXcmMessage) => {
                        frame_support::debug::print!("Receive Downward XCM: xcm = {:?}", xcm);
                    }
                    Err(e) => Self::deposit_event(XcmExecuteFail(hash, e)),
                };
            }
            Ok(Err(..)) => Self::deposit_event(XcmBadVersion(hash)),
            Err(..) => Self::deposit_event(XcmBadFormat(hash)),
        }
    }
}

impl<T: Config> HrmpMessageHandler for Module<T> {
    fn handle_hrmp_message(sender: ParaId, msg: InboundHrmpMessage) {
        let hash = T::Hashing::hash(&msg.data);
        frame_support::debug::print!("Processing HRMP XCM: {:?}", &hash);
        match VersionedXcm::decode(&mut &msg.data[..]).map(Xcm::try_from) {
            Ok(Ok(xcm)) => {
                sp_std::if_std! { println!("zenlink::<handle_hrmp_message> xcm {:?}", xcm); }
                let origin =
                    MultiLocation::X2(Junction::Parent, Junction::Parachain { id: sender.into() });
                match T::XcmExecutor::execute_xcm(origin, xcm) {
                    Ok(..) => Self::deposit_event(XcmExecuteSuccess(hash)),
                    Err(e) => Self::deposit_event(XcmExecuteFail(hash, e)),
                };
            }
            Ok(Err(..)) => Self::deposit_event(XcmBadVersion(hash)),
            Err(..) => Self::deposit_event(XcmBadFormat(hash)),
        }
    }
}

// TODO: more checks
fn shift_xcm(index: u8, msg: Xcm) -> Option<Xcm> {
    match msg {
        Xcm::ReserveAssetDeposit { assets, effects } => {
            let assets = assets
                .iter()
                .filter_map(|asset| match asset {
                    // In case 1: Asset'
                    //
                    // Asset (on chain A) -> Asset' (on chain B)
                    MultiAsset::ConcreteFungible {
                        id: MultiLocation::X2(Junction::Parent, Junction::Parachain { id }),
                        amount,
                    } => Some(MultiAsset::ConcreteFungible {
                        id: MultiLocation::X2(
                            Junction::PalletInstance { id: index },
                            Junction::GeneralIndex { id: *id as u128 },
                        ),
                        amount: *amount,
                    }),
                    // In case 2: Asset''
                    //
                    // Asset (on chain A) -> Asset' (on chain B), Asset' (on chain B) -> Asset'' (on chain C)
                    // In Actually, `Asset' (on chain B) -> Asset'' (on chain C)` is equal to:
                    // Asset' (on chain B) -> Asset (on chain A) -> Asset'' (on chain C)
                    MultiAsset::ConcreteFungible {
                        id: MultiLocation::X4(Junction::Parent, Junction::Parachain { .. }, Junction::PalletInstance { .. }, Junction::GeneralIndex { id }),
                        amount,
                    } => Some(MultiAsset::ConcreteFungible {
                        id: MultiLocation::X2(
                            Junction::PalletInstance { id: index },
                            Junction::GeneralIndex { id: *id },
                        ),
                        amount: *amount,
                    }),
                    _ => None,
                })
                .collect::<Vec<_>>();

            Some(Xcm::ReserveAssetDeposit { assets, effects })
        }
        Xcm::WithdrawAsset { .. } => Some(msg),
        _ => None,
    }
}

impl<T: Config> SendXcm for Module<T> {
    fn send_xcm(dest: MultiLocation, msg: Xcm) -> Result<(), XcmError> {
        let vmsg: VersionedXcm = msg.clone().into();
        sp_std::if_std! { println!("zenlink::<send_xcm> msg = {:?}, dest = {:?}", vmsg, dest); }
        match dest.first() {
            // A message for us. Execute directly.
            None => {
                let msg = vmsg.try_into().map_err(|_| XcmError::UnhandledXcmVersion)?;
                let res = T::XcmExecutor::execute_xcm(MultiLocation::Null, msg);
                sp_std::if_std! { println!("zenlink::<send_xcm>  res = {:?}", res); }
                res
            }
            // An upward message for the relay chain.
            Some(Junction::Parent) if dest.len() == 1 => {
                let data = vmsg.encode();
                let hash = T::Hashing::hash(&data);
                T::UpwardMessageSender::send_upward_message(data)
                    .map_err(|_| XcmError::Undefined)?;
                Self::deposit_event(UpwardMessageSent(hash));
                sp_std::if_std! { println!("zenlink::<send_xcm> upward success"); }
                Ok(())
            }
            // An HRMP message for a sibling parachain.
            Some(Junction::Parachain { id }) => {
                let data = vmsg.encode();
                let hash = T::Hashing::hash(&data);
                let message = OutboundHrmpMessage { recipient: (*id).into(), data };
                sp_std::if_std! { println!("zenlink::<send_xcm> X1 hrmp message = {:?}", message); }
                // TODO: Better error here
                T::HrmpMessageSender::send_hrmp_message(message)
                    .map_err(|_| XcmError::CannotReachDestination)?;
                Self::deposit_event(HrmpMessageSent(hash));
                sp_std::if_std! { println!("zenlink::<send_xcm> X1 hrmp success"); }
                Ok(())
            }
            // An HRMP message for a sibling parachain by zenlink
            Some(Junction::Parent) if dest.len() == 2 => {
                let vmsg: VersionedXcm = shift_xcm(Self::index(), msg)
                    .ok_or(XcmError::UnhandledXcmMessage)
                    .map(|m| m.into())?;
                match dest.at(1) {
                    Some(Junction::Parachain { id }) => {
                        let data = vmsg.encode();
                        let hash = T::Hashing::hash(&data);
                        let message = OutboundHrmpMessage { recipient: (*id).into(), data };

                        sp_std::if_std! { println!("zenlink::<send_xcm> X2 hrmp message = {:?}, data detail = {:?}", message, vmsg); }
                        // TODO: Better error here
                        T::HrmpMessageSender::send_hrmp_message(message)
                            .map_err(|_| XcmError::CannotReachDestination)?;
                        Self::deposit_event(HrmpMessageSent(hash));
                        sp_std::if_std! { println!("zenlink::<send_xcm> X2 hrmp success"); }
                        Ok(())
                    }
                    _ => {
                        sp_std::if_std! { println!("zenlink::<send_xcm> X2 UnhandledXcmMessage"); }
                        Err(XcmError::UnhandledXcmMessage)
                    }
                }
            }
            _ => {
                /* TODO: Handle other cases, like downward message */
                sp_std::if_std! { println!("zenlink::<send_xcm> UnhandledXcmMessage"); }
                Err(XcmError::UnhandledXcmMessage)
            }
        }
    }
}
