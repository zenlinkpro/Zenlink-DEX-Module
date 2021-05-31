// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

pub use zenlink_protocol::{
    make_x2_location, AssetBalance, AssetId, MultiAssetsHandler, PairInfo, TransactorAdaptor,
    TrustedParas, ZenlinkMultiAssets,
};

use super::{
    parameter_types, vec, AccountId, AccountId32, AccountId32Aliases, Balances, Event, Get,
    MultiLocation, NetworkId, PalletId, Parachain, ParachainInfo, Parent, Runtime, ShouldExecute,
    Sibling, SiblingParachainConvertsVia, Vec, Weight, Xcm, XcmConfig, XcmExecutor,
    ZenlinkProtocol, X1, X2,
};

parameter_types! {
    pub const ZenlinkPalletId: PalletId = PalletId(*b"/zenlink");
    pub const GetExchangeFee: (u32, u32) = (3, 1000);   // 0.3%
    pub SelfParaId: u32 = ParachainInfo::get().into();

    // xcm
    pub const AnyNetwork: NetworkId = NetworkId::Any;
    pub ZenlinkRegistedParaChains: Vec<(MultiLocation, u128)> = vec![
        // Bifrost local and live, 0.01 BNC
        (make_x2_location(2001), 10_000_000_000),
        // Phala local and live, 1 PHA
        (make_x2_location(2004), 1_000_000_000_000),
        // Plasm local and live, 0.0000000000001 SDN
        (make_x2_location(2007), 1_000_000),
        // Sherpax live, 0 KSX
        (make_x2_location(2013), 0),

        // Zenlink local 1 for test
        (make_x2_location(200), 1_000_000),
        // Zenlink local 2 for test
        (make_x2_location(300), 1_000_000),
    ];
}

pub struct ZenlinkAllowUnpaid<RegisteredChains>(PhantomData<RegisteredChains>);

impl<RegisteredChains> ShouldExecute for ZenlinkAllowUnpaid<RegisteredChains>
where
    RegisteredChains: Get<Vec<(MultiLocation, u128)>>,
{
    fn should_execute<Call>(
        origin: &MultiLocation,
        _top_level: bool,
        _message: &Xcm<Call>,
        _shallow_weight: Weight,
        _weight_credit: &mut Weight,
    ) -> Result<(), ()> {
        frame_support::log::info!("zenlink_protocol: ZenlinkAllowUnpaid = {:?}", origin);

        match origin {
            X1(AccountId32 { network, .. }) if *network == NetworkId::Any => Ok(()),
            X2(Parent, Parachain(id)) => {
                match RegisteredChains::get()
                    .iter()
                    .find(|(location, _)| *location == make_x2_location(*id))
                {
                    Some(_) => Ok(()),
                    None => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}

pub type ZenlinkLocationToAccountId = (
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<AnyNetwork, AccountId>,
);

use super::Assets;
use frame_support::dispatch::{DispatchError, DispatchResult};
use pallet_assets::ExternalAssetHandler;
use sp_std::marker::PhantomData;
use zenlink_protocol::LocalAssetHandler;

pub struct LocalAssetAdaptor<Local>(PhantomData<Local>);

impl<Local> LocalAssetHandler<AccountId> for LocalAssetAdaptor<Local>
where
    Local: ExternalAssetHandler<u32, AccountId, u128>,
{
    fn local_balance_of(asset_id: AssetId, who: &AccountId) -> AssetBalance {
        Local::balance(asset_id.asset_index, who.clone())
    }

    fn local_total_supply(asset_id: AssetId) -> AssetBalance {
        Local::total_supply(asset_id.asset_index)
    }

    fn local_is_exists(asset_id: AssetId) -> bool {
        Local::contains(asset_id.asset_index)
    }

    fn local_transfer(
        asset_id: AssetId,
        origin: &AccountId,
        target: &AccountId,
        amount: AssetBalance,
    ) -> DispatchResult {
        Local::transfer(asset_id.asset_index, origin.clone(), target.clone(), amount)
    }

    fn local_deposit(
        asset_id: AssetId,
        origin: &AccountId,
        amount: AssetBalance,
    ) -> Result<AssetBalance, DispatchError> {
        let _ = Local::deposit(asset_id.asset_index, origin.clone(), amount)?;

        Ok(amount)
    }

    fn local_withdraw(
        asset_id: AssetId,
        origin: &AccountId,
        amount: AssetBalance,
    ) -> Result<AssetBalance, DispatchError> {
        let _ = Local::withdraw(asset_id.asset_index, origin.clone(), amount)?;

        Ok(amount)
    }
}

type MultiAssets = ZenlinkMultiAssets<ZenlinkProtocol, Balances, LocalAssetAdaptor<Assets>>;

impl zenlink_protocol::Config for Runtime {
    type Event = Event;
    type GetExchangeFee = GetExchangeFee;
    type MultiAssetsHandler = MultiAssets;
    type PalletId = ZenlinkPalletId;
    type SelfParaId = SelfParaId;

    type TargetChains = ZenlinkRegistedParaChains;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type Conversion = ZenlinkLocationToAccountId;
}
