// Copyright 2021-2022 Zenlink.
// Licensed under Apache 2.0.

//! Test utilities

#[cfg(feature = "std")]
use std::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	pallet_prelude::GenesisBuild,
	parameter_types,
	traits::{ConstU32, Contains},
	PalletId,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	RuntimeDebug,
};

use crate as pallet_zenlink;
pub use crate::{
	AssetBalance, AssetId, AssetIdConverter, Config, MultiAssetsHandler, PairLpGenerate, Pallet,
	ParaId, ZenlinkMultiAssets, LIQUIDITY, LOCAL, NATIVE, RESERVED,
};
use orml_traits::{parameter_type_with_key, MultiCurrency};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

#[derive(
	Encode,
	Decode,
	Eq,
	PartialEq,
	Copy,
	Clone,
	RuntimeDebug,
	PartialOrd,
	MaxEncodedLen,
	Ord,
	TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(u8),
	ZenlinkLp(u8, u8),
}

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 8,
		Zenlink: pallet_zenlink::{Pallet, Call, Storage, Event<T>} = 9,
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>} = 11,
	}
);

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;

	pub const BlockHashCount: u64 = 250;
	pub const ZenlinkPalletId: PalletId = PalletId(*b"/zenlink");
	pub const MaxReserves: u32 = 50;
	pub const MaxLocks:u32 = 50;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
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
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> u128 {
		0
	};
}

pub struct MockDustRemovalWhitelist;
impl Contains<AccountId> for MockDustRemovalWhitelist {
	fn contains(_a: &AccountId) -> bool {
		true
	}
}

pub type ReserveIdentifier = [u8; 8];

impl orml_tokens::Config for Test {
	type Event = Event;
	type Balance = u128;
	type Amount = i128;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = MockDustRemovalWhitelist;
	type ReserveIdentifier = ReserveIdentifier;
	type MaxReserves = ConstU32<100_000>;
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

impl Config for Test {
	type Event = Event;
	type MultiAssetsHandler = ZenlinkMultiAssets<Zenlink, Balances, AssetAdaptor<Tokens>>;
	type PalletId = ZenlinkPalletId;
	type AssetId = AssetId;
	type LpGenerate = PairLpGenerate<Self>;
	type TargetChains = ();
	type SelfParaId = ();
	type XcmExecutor = ();
	type AccountIdConverter = ();
	type AssetIdConverter = AssetIdConverter;
	type WeightInfo = ();
}

pub type DexPallet = Pallet<Test>;

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

	orml_tokens::GenesisConfig::<Test> {
		balances: vec![
			(1, CurrencyId::Token(1), 34028236692093846346337460743176821145),
			(1, CurrencyId::Token(2), 34028236692093846346337460743176821145),
			(1, CurrencyId::Token(3), 34028236692093846346337460743176821145),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_zenlink::GenesisConfig::<Test> { fee_receiver: None, fee_point: 5 }
		.assimilate_storage(&mut t)
		.unwrap();

	t.into()
}

pub struct AssetAdaptor<Local>(PhantomData<Local>);

type AccountId = u128;

fn asset_id_to_currency_id(asset_id: &AssetId) -> Result<CurrencyId, ()> {
	let discr = (asset_id.asset_index & 0x0000_0000_0000_ff00) >> 8;
	return if discr == 6 {
		let token0_id = ((asset_id.asset_index & 0x0000_0000_ffff_0000) >> 16) as u8;
		let token1_id = ((asset_id.asset_index & 0x0000_ffff_0000_0000) >> 16) as u8;
		Ok(CurrencyId::ZenlinkLp(token0_id, token1_id))
	} else {
		let token_id = asset_id.asset_index as u8;

		Ok(CurrencyId::Token(token_id))
	}
}

impl<Local> MultiCurrency<AccountId> for AssetAdaptor<Local>
where
	Local: MultiCurrency<AccountId, Balance = u128, CurrencyId = CurrencyId>,
{
	type Balance = u128;
	type CurrencyId = AssetId;

	fn minimum_balance(asset_id: Self::CurrencyId) -> Self::Balance {
		asset_id_to_currency_id(&asset_id)
			.map_or(AssetBalance::default(), |currency_id| Local::minimum_balance(currency_id))
	}

	fn total_issuance(asset_id: Self::CurrencyId) -> Self::Balance {
		asset_id_to_currency_id(&asset_id)
			.map_or(AssetBalance::default(), |currency_id| Local::total_issuance(currency_id))
	}

	fn total_balance(asset_id: Self::CurrencyId, who: &AccountId) -> Self::Balance {
		asset_id_to_currency_id(&asset_id)
			.map_or(AssetBalance::default(), |currency_id| Local::total_balance(currency_id, who))
	}

	fn free_balance(asset_id: Self::CurrencyId, who: &AccountId) -> Self::Balance {
		asset_id_to_currency_id(&asset_id)
			.map_or(AssetBalance::default(), |currency_id| Local::free_balance(currency_id, who))
	}

	fn ensure_can_withdraw(
		currency_id: Self::CurrencyId,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		asset_id_to_currency_id(&asset_id).map_or(Err(DispatchError::CannotLookup), |currency_id| {
			Local::ensure_can_withdraw(currency_id, who, amount)
		})
	}

	fn transfer(
		currency_id: Self::CurrencyId,
		from: &AccountId,
		to: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		asset_id_to_currency_id(&asset_id).map_or(Err(DispatchError::CannotLookup), |currency_id| {
			Local::transfer(currency_id, from, to, amount)
		})
	}

	fn deposit(
		currency_id: Self::CurrencyId,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		asset_id_to_currency_id(&asset_id).map_or(Err(DispatchError::CannotLookup), |currency_id| {
			Local::deposit(currency_id, who, amount)
		})
	}

	fn withdraw(
		currency_id: Self::CurrencyId,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		asset_id_to_currency_id(&asset_id).map_or(Err(DispatchError::CannotLookup), |currency_id| {
			Local::withdraw(currency_id, who, amount)
		})
	}

	fn can_slash(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> bool {
		asset_id_to_currency_id(&asset_id).map_or(Err(DispatchError::CannotLookup), |currency_id| {
			Local::can_slash(currency_id, who, value)
		})
	}

	fn slash(
		currency_id: Self::CurrencyId,
		who: &AccountId,
		amount: Self::Balance,
	) -> Self::Balance {
		asset_id_to_currency_id(&asset_id).map_or(Err(DispatchError::CannotLookup), |currency_id| {
			Local::slash(currency_id, who, amount)
		})
	}
}
