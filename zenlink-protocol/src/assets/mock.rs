// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! Test utilities
use frame_support::parameter_types;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use crate as pallet_zenlink;
use crate::{
    Config, HrmpMessageSender, Module, ModuleId, OperationalAsset, OutboundHrmpMessage,
    UpwardMessage, UpwardMessageSender,
};
use frame_support::dispatch::DispatchResult;

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
    pub const ExistentialDeposit: u64 = 1;
    pub const TestModuleId: ModuleId = ModuleId(*b"zenlink1");
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Index = u64;
    type Call = Call;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
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

impl pallet_balances::Config for Test {
    type Balance = u128;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
    type WeightInfo = ();
    type MaxLocks = ();
}
pub struct TestAssets;

impl OperationalAsset<u32, u64, u128> for TestAssets {
    fn module_index() -> u8 {
        unimplemented!()
    }

    fn balance(_id: u32, _who: u64) -> u128 {
        unimplemented!()
    }

    fn total_supply(_id: u32) -> u128 {
        unimplemented!()
    }

    fn inner_transfer(_id: u32, _origin: u64, _target: u64, _amount: u128) -> DispatchResult {
        unimplemented!()
    }

    fn inner_deposit(_id: u32, _origin: u64, _amount: u128) -> DispatchResult {
        unimplemented!()
    }

    fn inner_withdraw(_id: u32, _origin: u64, _amount: u128) -> DispatchResult {
        unimplemented!()
    }
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

impl Config for Test {
    type Event = Event;
    type NativeCurrency = pallet_balances::Module<Test>;
    type XcmExecutor = ();
    type UpwardMessageSender = TestSender;
    type HrmpMessageSender = TestSender;
    type AccountIdConverter = ();
    type AccountId32Converter = ();
    type ModuleId = TestModuleId;
    type ParaId = ();
    type TargetChains = ();
    type OperationalAsset = TestAssets;
}

pub type Assets = Module<Test>;
pub(crate) const CURRENCY_AMOUNT: u128 = 1000;
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, CURRENCY_AMOUNT),
            (2, CURRENCY_AMOUNT),
            (3, CURRENCY_AMOUNT),
            (4, CURRENCY_AMOUNT),
            (5, CURRENCY_AMOUNT),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}
