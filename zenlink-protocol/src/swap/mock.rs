// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! Test utilities
use frame_support::{parameter_types, Hashable};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use crate as pallet_zenlink;
use crate::{
    Config, Convert, ExecuteXcm, HrmpMessageSender, LocationConversion, Module, ModuleId,
    MultiLocation, OutboundHrmpMessage, UpwardMessage, UpwardMessageSender, Xcm, XcmResult,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>} = 0,
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>} = 8,
        Zenlink: pallet_zenlink::{Module, Origin, Call, Storage, Event<T>} = 9,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const DEXModuleId: ModuleId = ModuleId(*b"zenlink1");
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Index = u64;
    type Call = Call;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u128;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type PalletInfo = PalletInfo;
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type Balance = u128;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
    type WeightInfo = ();
    type MaxLocks = ();
}

pub struct TestSender;

impl UpwardMessageSender for TestSender {
    fn send_upward_message(_msg: UpwardMessage) -> Result<(), ()> {
        unimplemented!()
    }
}

impl HrmpMessageSender for TestSender {
    /// Send the given HRMP message.
    fn send_hrmp_message(_msg: OutboundHrmpMessage) -> Result<(), ()> {
        unimplemented!()
    }
}

impl ExecuteXcm for TestSender {
    fn execute_xcm(_origin: MultiLocation, _msg: Xcm) -> XcmResult {
        Ok(())
    }
}

pub struct Converter;
impl Convert<<Test as frame_system::Config>::AccountId, [u8; 32]> for Converter {
    fn convert(account_id: <Test as frame_system::Config>::AccountId) -> [u8; 32] {
        account_id.twox_256()
    }
}

impl LocationConversion<<Test as frame_system::Config>::AccountId> for Converter {
    fn from_location(_location: &MultiLocation) -> Option<u128> {
        Some(0u128)
    }

    fn try_into_location(_who: u128) -> Result<MultiLocation, u128> {
        Ok(MultiLocation::Null)
    }
}

impl Config for Test {
    type Event = Event;
    type XcmExecutor = TestSender;
    type UpwardMessageSender = TestSender;
    type HrmpMessageSender = TestSender;
    type AccountIdConverter = Converter;
    type AccountId32Converter = Converter;
    type ModuleId = DEXModuleId;
    type ParaId = ();
    type TargetChains = ();
    type AssetModuleRegistry = ();
}

pub type DexModule = Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 34028236692093846346337460743176821145),
            (2, 10),
            (3, 10),
            (4, 10),
            (5, 10),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}
