use super::{
    parameter_types, vec, AccountId, Balances, Call, Convert, DispatchResult, Event, ModuleId,
    Origin, ParachainInfo, ParachainSystem, Runtime, Vec, ZenlinkProtocol,
};

use zenlink_protocol::{
    AccountId32Aliases, Junction, LocationInverter, MultiLocation, NetworkId, OperationalAsset,
    Origin as ZenlinkOrigin, ParaChainWhiteList, ParentIsDefault, RelayChainAsNative, Sibling,
    SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
    SovereignSignedViaLocation, TokenBalance, Transactor, XcmCfg, XcmExecutor,
};

parameter_types! {
    pub const RococoNetwork: NetworkId = NetworkId::Polkadot;
    pub const DEXModuleId: ModuleId = ModuleId(*b"zenlink1");
    pub RelayChainOrigin: Origin = ZenlinkOrigin::Relay.into();
    pub Ancestry: MultiLocation = Junction::Parachain {
        id: ParachainInfo::parachain_id().into()
    }.into();

    pub SiblingParachains: Vec<MultiLocation> = vec![
        // Sherpax live
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 59 }),
        // Bifrost local and live
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 107 }),
        // Zenlink live
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 188 }),
        // Zenlink local
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 200 }),
        // Sherpax local
        MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 300 })
    ];
}

pub struct AccountId32Converter;
impl Convert<AccountId, [u8; 32]> for AccountId32Converter {
    fn convert(account_id: AccountId) -> [u8; 32] {
        account_id.into()
    }
}

type LocationConverter = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RococoNetwork, AccountId>,
);

pub type LocalAssetTransactor =
    Transactor<Balances, ZenlinkProtocol, LocationConverter, AccountId, ParachainInfo>;

type LocalOriginConverter = (
    SovereignSignedViaLocation<LocationConverter, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<ZenlinkOrigin, Origin>,
    SignedAccountId32AsNative<RococoNetwork, Origin>,
);

pub struct XcmConfig;

impl XcmCfg for XcmConfig {
    type Call = Call;
    type XcmSender = ZenlinkProtocol;
    // How to withdraw and deposit an asset.
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = LocalOriginConverter;
    type IsReserve = ParaChainWhiteList<SiblingParachains>;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
}

pub struct OtherAssets;
impl OperationalAsset<u32, AccountId, TokenBalance> for OtherAssets {
    fn module_index() -> u8 {
        unimplemented!()
    }
    /// Get the asset `id` balance of `who`.
    fn balance(_id: u32, _who: AccountId) -> TokenBalance {
        unimplemented!()
    }

    /// Get the total supply of an asset `id`.
    fn total_supply(_id: u32) -> TokenBalance {
        unimplemented!()
    }

    fn inner_transfer(
        _id: u32,
        _origin: AccountId,
        _target: AccountId,
        _amount: TokenBalance,
    ) -> DispatchResult {
        unimplemented!()
    }

    fn inner_deposit(_id: u32, _origin: AccountId, _amount: TokenBalance) -> DispatchResult {
        unimplemented!()
    }

    fn inner_withdraw(_id: u32, _origin: AccountId, _amount: TokenBalance) -> DispatchResult {
        unimplemented!()
    }
}

impl zenlink_protocol::Config for Runtime {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type UpwardMessageSender = ParachainSystem;
    type HrmpMessageSender = ParachainSystem;
    type NativeCurrency = Balances;
    type AccountIdConverter = LocationConverter;
    type AccountId32Converter = AccountId32Converter;
    type ParaId = ParachainInfo;
    type ModuleId = DEXModuleId;
    type TargetChains = SiblingParachains;
    type OperationalAsset = OtherAssets;
}
