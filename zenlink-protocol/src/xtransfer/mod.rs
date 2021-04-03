// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

use codec::{Decode, Encode};
use sp_runtime::traits::Hash;
use sp_std::{
    convert::{TryFrom, TryInto},
    vec, prelude::Vec,
};

use crate::{
    sp_api_hidden_includes_decl_storage::hidden_include::traits::PalletInfo,
    AssetId, AssetProperty, Config, Convert, DownwardMessageHandler, ExecuteXcm, Get,
    XcmpMessageHandler, XcmpMessageSender, InboundDownwardMessage, Junction,
    Module, MultiAsset, MultiLocation, NetworkId, Order, ParaId,
    RawEvent::{
        HrmpMessageSent, UpwardMessageSent, XcmBadFormat, XcmBadVersion, XcmExecuteFail,
        XcmExecuteSuccess,
    },
    SendXcm, TokenBalance, UpwardMessageSender, VersionedXcm, Xcm, XcmError, LOG_TARGET,
    relay_chain, ServiceQuality,
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
    pub(crate) fn index() -> u8 {
        T::PalletInfo::index::<Self>().map_or(0u8, |index| index as u8)
    }

    pub(crate) fn is_reachable(para_id: ParaId) -> bool {
        let location =
            MultiLocation::X2(Junction::Parent, Junction::Parachain { id: para_id.into() });

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
    ) -> Result<Xcm, XcmError> {
        let asset_location = MultiLocation::X4(
            Junction::Parent,
            Junction::Parachain { id: asset_id.chain_id },
            Junction::PalletInstance { id: asset_id.module_index },
            Junction::GeneralIndex { id: asset_id.asset_index as u128 },
        );
        if Self::assets_list().contains(asset_id) {
            match Self::asset_property(asset_id) {
                AssetProperty::Foreign => Ok(Self::make_xcm_lateral_transfer_foreign(
                    ParaId::from(asset_id.chain_id),
                    asset_location,
                    para_id,
                    account.clone(),
                    amount,
                )),
                AssetProperty::Lp(_) => Ok(Self::make_xcm_lateral_transfer_native(
                    asset_location,
                    para_id,
                    account.clone(),
                    amount,
                )),
            }
        } else {
            T::AssetModuleRegistry::get()
                .iter()
                .find(|(index, _)| {
                    *index == asset_id.module_index
                        && <T as Config>::ParaId::get() == ParaId::from(asset_id.chain_id)
                })
                .map_or(
                    Err(XcmError::FailedToTransactAsset("No match asset by the asset id")),
                    |(_, _)| {
                        Ok(Self::make_xcm_lateral_transfer_native(
                            asset_location,
                            para_id,
                            account.clone(),
                            amount,
                        ))
                    },
                )
        }
    }
}

impl<T: Config> DownwardMessageHandler for Module<T> {
    fn handle_downward_message(msg: InboundDownwardMessage) {
        let hash = msg.using_encoded(T::Hashing::hash);
        log::info!(target: LOG_TARGET, "Processing Downward XCM: hash = {:?}", &hash);
        match VersionedXcm::decode(&mut &msg.msg[..]).map(Xcm::try_from) {
            Ok(Ok(xcm)) => {
                match T::XcmExecutor::execute_xcm(Junction::Parent.into(), xcm.clone()) {
                    Ok(..) => Self::deposit_event(XcmExecuteSuccess(hash)),
                    Err(_e @ XcmError::UnhandledXcmMessage) => {
                        log::info!(target: LOG_TARGET, "handle_dmp_message: xcm = {:?}", xcm);
                    }
                    Err(e) => Self::deposit_event(XcmExecuteFail(hash, e)),
                };
            }
            Ok(Err(..)) => Self::deposit_event(XcmBadVersion(hash)),
            Err(..) => Self::deposit_event(XcmBadFormat(hash)),
        }
    }
}

impl<T: Config> XcmpMessageHandler for Module<T> {
    fn handle_xcm_message(sender: ParaId, _sent_at: relay_chain::BlockNumber, xcm: VersionedXcm) {
        let hash = xcm.using_encoded(T::Hashing::hash);
        log::info!(target: LOG_TARGET, "Processing HRMP XCM: {:?}", &hash);
        match Xcm::try_from(xcm) {
            Ok(xcm) => {
                log::info!(target: LOG_TARGET, "handle_xcm_message: xcm = {:?}", xcm);

                let origin =
                    MultiLocation::X2(Junction::Parent, Junction::Parachain { id: sender.into() });

                match T::XcmExecutor::execute_xcm(origin, xcm) {
                    Ok(..) => Self::deposit_event(XcmExecuteSuccess(hash)),
                    Err(e) => Self::deposit_event(XcmExecuteFail(hash, e)),
                };
            }
            Err(..) => Self::deposit_event(XcmBadVersion(hash)),
        };
    }

    fn handle_blob_message(_sender: ParaId, _sent_at: relay_chain::BlockNumber, _blob: Vec<u8>) {
        debug_assert!(false, "Blob messages not handled.")
    }
}

impl<T: Config> SendXcm for Module<T> {
    fn send_xcm(dest: MultiLocation, msg: Xcm) -> Result<(), XcmError> {
        let vmsg: VersionedXcm = msg.clone().into();
        log::info!(target: LOG_TARGET, "send_xcm: msg = {:?}, dest = {:?}", vmsg, dest);

        match dest.first() {
            // A message for us. Execute directly.
            None => {
                let msg = vmsg.try_into().map_err(|_| XcmError::UnhandledXcmVersion)?;

                #[warn(clippy::let_and_return)]
                let res = T::XcmExecutor::execute_xcm(MultiLocation::Null, msg);

                log::debug!(target: LOG_TARGET, "send_xcm(for us): executed result = {:?}", res);

                res
            }
            // An upward message for the relay chain.
            Some(Junction::Parent) if dest.len() == 1 => {
                let data = vmsg.encode();
                let hash = T::Hashing::hash(&data);

                T::UpwardMessageSender::send_upward_message(data)
                    .map_err(|_| XcmError::CannotReachDestination)?;

                Self::deposit_event(UpwardMessageSent(hash));

                log::debug!(target: LOG_TARGET, "send_xcm(ump): success");

                Ok(())
            }
            // An HRMP message for a sibling parachain.
            Some(Junction::Parachain { id }) => {
                let hash = T::Hashing::hash_of(&vmsg);
                T::XcmpMessageSender::send_xcm_message((*id).into(), vmsg, ServiceQuality::Ordered)
                    .map_err(|_| XcmError::CannotReachDestination)?;

                Self::deposit_event(HrmpMessageSent(hash));

                log::debug!(target: LOG_TARGET, "send_xcm(x1 hrmp): success");

                Ok(())
            }
            // An HRMP message for a sibling parachain by zenlink
            Some(Junction::Parent) if dest.len() == 2 => {
                let vmsg: VersionedXcm = msg.into();
                match dest.at(1) {
                    Some(Junction::Parachain { id }) => {
                        let hash = T::Hashing::hash_of(&vmsg);

                        T::XcmpMessageSender::send_xcm_message((*id).into(), vmsg, ServiceQuality::Ordered)
                            .map_err(|_| XcmError::CannotReachDestination)?;

                        Self::deposit_event(HrmpMessageSent(hash));

                        log::debug!(target: LOG_TARGET, "send_xcm(x2 hrmp): success");

                        Ok(())
                    }
                    _ => {
                        log::debug!(target: LOG_TARGET, "send_xcm(x2 hrmp): unhandled");

                        Err(XcmError::UnhandledXcmMessage)
                    }
                }
            }
            _ => {
                log::debug!(target: LOG_TARGET, "send_xcm(dmp or other): unhandled");

                Err(XcmError::UnhandledXcmMessage)
            }
        }
    }
}
