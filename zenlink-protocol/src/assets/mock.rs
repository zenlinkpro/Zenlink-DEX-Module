use frame_support::{impl_outer_origin, parameter_types};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    ModuleId,
};

use crate::{
    Config, HrmpMessageSender, Module, OutboundHrmpMessage, UpwardMessage, UpwardMessageSender,
};

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const ExistentialDeposit: u64 = 1;
	pub const TestModuleId: ModuleId = ModuleId(*b"zlk_dex1");
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Index = u64;
    type Call = ();
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type PalletInfo = ();
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = ();
}

impl pallet_balances::Config for Test {
    type Balance = u128;
    type DustRemoval = ();
    type Event = ();
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

impl Config for Test {
    type Event = ();
    type NativeCurrency = pallet_balances::Module<Test>;
    type XcmExecutor = ();
    type UpwardMessageSender = TestSender;
    type HrmpMessageSender = TestSender;
    type AccountIdConverter = ();
    type AccountId32Converter = ();
    type ModuleId = TestModuleId;
    type ParaId = ();
}

pub type Assets = Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
