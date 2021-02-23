// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! Test utilities
use crate as pallet_zenlink;

use frame_support::parameter_types;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use crate::{
	Config, HrmpMessageSender, Module, ModuleId, OutboundHrmpMessage, UpwardMessage,
	UpwardMessageSender,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		Zenlink: pallet_zenlink::{Module, Origin, Call, Storage, Event<T>},
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
}

pub type Assets = Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
