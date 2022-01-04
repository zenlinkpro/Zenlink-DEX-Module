// Copyright 2020-2021 Zenlink
// Licensed under GPL-3.0.

//! Test utilities

#[cfg(feature = "std")]
use std::marker::PhantomData;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	pallet_prelude::GenesisBuild,
	parameter_types,
	traits::Contains,
	PalletId,
};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature, RuntimeDebug,
};

use crate as pallet_zenlink;
pub use crate::{
	AssetBalance, AssetId, Config, LocalAssetHandler, MultiAssetsHandler, Pallet, ParaId, ZenlinkMultiAssets,
	LIQUIDITY, LOCAL, NATIVE, RESERVED,
};
use orml_traits::{parameter_type_with_key, MultiCurrency};
use sp_core::{crypto::AccountId32, sr25519, Pair, Public, H256};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, TypeInfo)]
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
	type AccountId = AccountId32;
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
	type MultiAssetsHandler = ZenlinkMultiAssets<Zenlink, Balances, LocalAssetAdaptor<Tokens>>;
	type PalletId = ZenlinkPalletId;
	type TargetChains = ();
	type SelfParaId = ();
	type XcmExecutor = ();
	type Conversion = ();
	type WeightInfo = ();
}

pub type DexPallet = Pallet<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(alice(), 34028236692093846346337460743176821145)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	orml_tokens::GenesisConfig::<Test> {
		balances: vec![
			(alice(), CurrencyId::Token(1), 34028236692093846346337460743176821145),
			(alice(), CurrencyId::Token(2), 34028236692093846346337460743176821145),
			(alice(), CurrencyId::Token(3), 34028236692093846346337460743176821145),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_zenlink::GenesisConfig::<Test> {
		fee_receiver: None,
		fee_point: 5,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t.into()
}

pub struct LocalAssetAdaptor<Local>(PhantomData<Local>);

type AccountId = AccountId32;

fn asset_id_to_currency_id(asset_id: &AssetId) -> Result<CurrencyId, ()> {
	let discr = (asset_id.asset_index & 0x0000_0000_0000_ff00) >> 8;
	return if discr == 6 {
		let token0_id = ((asset_id.asset_index & 0x0000_0000_ffff_0000) >> 16) as u8;
		let token1_id = ((asset_id.asset_index & 0x0000_ffff_0000_0000) >> 16) as u8;
		Ok(CurrencyId::ZenlinkLp(token0_id, token1_id))
	} else {
		let token_id = asset_id.asset_index as u8;

		Ok(CurrencyId::Token(token_id))
	};
}

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

type AccountPublic = <MultiSignature as Verify>::Signer;

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn alice() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Alice")
}

pub fn bob() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Bob")
}
