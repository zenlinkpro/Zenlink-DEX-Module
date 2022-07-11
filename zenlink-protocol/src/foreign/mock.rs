// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! Test utilities
#[cfg(feature = "std")]
use std::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
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
	AssetBalance, AssetId, Config, LocalAssetHandler, MultiAssetsHandler, OtherAssetHandler, Pallet, ParaId,
	ZenlinkMultiAssets, LIQUIDITY, LOCAL, NATIVE, RESERVED,
};

use orml_traits::{parameter_type_with_key, MultiCurrency};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

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
	type MultiAssetsHandler = ZenlinkMultiAssets<Zenlink, Balances, LocalAssetAdaptor<Tokens>, MockOtherAsset>;
	type PalletId = ZenlinkPalletId;
	type TargetChains = ();
	type SelfParaId = ();
	type XcmExecutor = ();
	type Conversion = ();
	type WeightInfo = ();
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, MaxEncodedLen, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(u8),
	ZenlinkLp(u8, u8),
}

fn asset_id_to_currency_id(asset_id: &AssetId) -> Result<CurrencyId, ()> {
	if asset_id.asset_type == LIQUIDITY {
		let token0_id = ((asset_id.asset_index & 0xFFFF0000) >> 16) as u8;
		let token1_id = (asset_id.asset_index & 0x0000FFFF) as u8;
		return Ok(CurrencyId::ZenlinkLp(token0_id, token1_id));
	}
	if asset_id.asset_type == LOCAL {
		let token_id = asset_id.asset_index as u8;
		return Ok(CurrencyId::Token(token_id));
	}
	Err(())
}

pub struct LocalAssetAdaptor<Local>(PhantomData<Local>);

type AccountId = u128;

impl<Local> LocalAssetHandler<AccountId> for LocalAssetAdaptor<Local>
where
	Local: MultiCurrency<AccountId, Balance = u128, CurrencyId = CurrencyId>,
{
	fn local_balance_of(asset_id: AssetId, who: &AccountId) -> AssetBalance {
		asset_id_to_currency_id(&asset_id).map_or(AssetBalance::default(), |currency_id| {
			Local::free_balance(currency_id, who)
		})
	}

	fn local_total_supply(asset_id: AssetId) -> AssetBalance {
		asset_id_to_currency_id(&asset_id).map_or(AssetBalance::default(), |currency_id| {
			Local::total_issuance(currency_id)
		})
	}

	fn local_is_exists(asset_id: AssetId) -> bool {
		asset_id_to_currency_id(&asset_id).map_or(false, |currency_id| {
			Local::total_issuance(currency_id) > AssetBalance::default()
		})
	}

	fn local_transfer(
		asset_id: AssetId,
		origin: &AccountId,
		target: &AccountId,
		amount: AssetBalance,
	) -> DispatchResult {
		asset_id_to_currency_id(&asset_id).map_or(Err(DispatchError::CannotLookup), |currency_id| {
			Local::transfer(currency_id, origin, target, amount)
		})
	}

	fn local_deposit(
		asset_id: AssetId,
		origin: &AccountId,
		amount: AssetBalance,
	) -> Result<AssetBalance, DispatchError> {
		asset_id_to_currency_id(&asset_id).map_or(Ok(AssetBalance::default()), |currency_id| {
			Local::deposit(currency_id, origin, amount).map(|_| amount)
		})
	}

	fn local_withdraw(
		asset_id: AssetId,
		origin: &AccountId,
		amount: AssetBalance,
	) -> Result<AssetBalance, DispatchError> {
		asset_id_to_currency_id(&asset_id).map_or(Ok(AssetBalance::default()), |currency_id| {
			Local::withdraw(currency_id, origin, amount).map(|_| amount)
		})
	}
}

pub type DexPallet = Pallet<Test>;

pub struct MockOtherAsset;

impl<AccountId> OtherAssetHandler<AccountId> for MockOtherAsset {
	fn other_balance_of(_asset_id: AssetId, _who: &AccountId) -> AssetBalance {
		Default::default()
	}

	fn other_total_supply(_asset_id: AssetId) -> AssetBalance {
		Default::default()
	}

	fn other_is_exists(_asset_id: AssetId) -> bool {
		false
	}

	fn other_deposit(
		_asset_id: AssetId,
		_origin: &AccountId,
		_amount: AssetBalance,
	) -> Result<AssetBalance, DispatchError> {
		unimplemented!()
	}

	fn other_withdraw(
		_asset_id: AssetId,
		_origin: &AccountId,
		_amount: AssetBalance,
	) -> Result<AssetBalance, DispatchError> {
		unimplemented!()
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into();
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
