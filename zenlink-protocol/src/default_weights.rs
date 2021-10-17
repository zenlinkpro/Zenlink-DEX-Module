// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for zenlink_protocol
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-10-14, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("benchmarks"), DB CACHE: 128

// Executed Command:
// ./target/release/dev-parachain
// benchmark
// --chain
// benchmarks
// --steps=50
// --repeat=20
// --pallet=zenlink_protocol
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./zenlink-protocol/src/default_weights.rs
// --template=./.maintain/frame-weight-template.hbs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for zenlink_protocol.
pub trait WeightInfo {
	fn set_fee_receiver() -> Weight;
	fn set_fee_point() -> Weight;
	fn create_pair() -> Weight;
	fn bootstrap_create() -> Weight;
	fn bootstrap_contribute() -> Weight;
	fn bootstrap_claim() -> Weight;
	fn bootstrap_end() -> Weight;
	fn bootstrap_update() -> Weight;
	fn bootstrap_refund() -> Weight;
	fn add_liquidity() -> Weight;
	fn remove_liquidity() -> Weight;
	fn swap_exact_assets_for_assets() -> Weight;
	fn swap_assets_for_exact_assets() -> Weight;
}

/// Weights for zenlink_protocol using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:1)
	fn set_fee_receiver() -> Weight {
		(6_845_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:1)
	fn set_fee_point() -> Weight {
		(5_987_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens TotalIssuance (r:2 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: ZenlinkProtocol LiquidityPairs (r:0 w:1)
	fn create_pair() -> Weight {
		(38_251_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(8 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_create() -> Weight {
		(23_390_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: System Account (r:1 w:1)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_contribute() -> Weight {
		(132_175_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().writes(9 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:0)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapEndStatus (r:1 w:0)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_claim() -> Weight {
		(97_310_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: System Account (r:2 w:1)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: ZenlinkProtocol LiquidityPairs (r:0 w:1)
	// Storage: ZenlinkProtocol BootstrapEndStatus (r:0 w:1)
	fn bootstrap_end() -> Weight {
		(182_207_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(14 as Weight))
			.saturating_add(T::DbWeight::get().writes(12 as Weight))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_update() -> Weight {
		(26_462_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: System Account (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_refund() -> Weight {
		(109_743_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(12 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ZenlinkProtocol KLast (r:1 w:1)
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:0)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn add_liquidity() -> Weight {
		(208_373_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(16 as Weight))
			.saturating_add(T::DbWeight::get().writes(11 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ZenlinkProtocol KLast (r:1 w:1)
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:0)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: System Account (r:1 w:0)
	fn remove_liquidity() -> Weight {
		(168_013_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(16 as Weight))
			.saturating_add(T::DbWeight::get().writes(10 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:6 w:6)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: ZenlinkProtocol PairStatuses (r:2 w:0)
	// Storage: System Account (r:2 w:0)
	fn swap_exact_assets_for_assets() -> Weight {
		(191_434_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(15 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:6 w:6)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: ZenlinkProtocol PairStatuses (r:2 w:0)
	// Storage: System Account (r:2 w:0)
	fn swap_assets_for_exact_assets() -> Weight {
		(192_004_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(15 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:1)
	fn set_fee_receiver() -> Weight {
		(6_845_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:1)
	fn set_fee_point() -> Weight {
		(5_987_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(1 as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens TotalIssuance (r:2 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: ZenlinkProtocol LiquidityPairs (r:0 w:1)
	fn create_pair() -> Weight {
		(38_251_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(8 as Weight))
			.saturating_add(RocksDbWeight::get().writes(4 as Weight))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_create() -> Weight {
		(23_390_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5 as Weight))
			.saturating_add(RocksDbWeight::get().writes(3 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: System Account (r:1 w:1)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_contribute() -> Weight {
		(132_175_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(12 as Weight))
			.saturating_add(RocksDbWeight::get().writes(9 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:0)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ZenlinkProtocol BootstrapEndStatus (r:1 w:0)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:2 w:2)
	// Storage: System Account (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_claim() -> Weight {
		(97_310_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(12 as Weight))
			.saturating_add(RocksDbWeight::get().writes(5 as Weight))
	}
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: System Account (r:2 w:1)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: ZenlinkProtocol LiquidityPairs (r:0 w:1)
	// Storage: ZenlinkProtocol BootstrapEndStatus (r:0 w:1)
	fn bootstrap_end() -> Weight {
		(182_207_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(14 as Weight))
			.saturating_add(RocksDbWeight::get().writes(12 as Weight))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_update() -> Weight {
		(26_462_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(5 as Weight))
			.saturating_add(RocksDbWeight::get().writes(3 as Weight))
	}
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol BootstrapPersonalSupply (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: System Account (r:1 w:0)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	fn bootstrap_refund() -> Weight {
		(109_743_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(12 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ZenlinkProtocol KLast (r:1 w:1)
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:0)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn add_liquidity() -> Weight {
		(208_373_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(16 as Weight))
			.saturating_add(RocksDbWeight::get().writes(11 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ZenlinkProtocol PairStatuses (r:1 w:1)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: ZenlinkProtocol LiquidityPairs (r:1 w:0)
	// Storage: ZenlinkProtocol KLast (r:1 w:1)
	// Storage: ZenlinkProtocol FeeMeta (r:1 w:0)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: System Account (r:1 w:0)
	fn remove_liquidity() -> Weight {
		(168_013_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(16 as Weight))
			.saturating_add(RocksDbWeight::get().writes(10 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:6 w:6)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: ZenlinkProtocol PairStatuses (r:2 w:0)
	// Storage: System Account (r:2 w:0)
	fn swap_exact_assets_for_assets() -> Weight {
		(191_434_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(15 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
	// Storage: System Number (r:1 w:0)
	// Storage: ParachainInfo ParachainId (r:1 w:0)
	// Storage: Tokens Accounts (r:6 w:6)
	// Storage: System ExecutionPhase (r:1 w:0)
	// Storage: System EventCount (r:1 w:1)
	// Storage: System Events (r:1 w:1)
	// Storage: ZenlinkProtocol PairStatuses (r:2 w:0)
	// Storage: System Account (r:2 w:0)
	fn swap_assets_for_exact_assets() -> Weight {
		(192_004_000 as Weight)
			.saturating_add(RocksDbWeight::get().reads(15 as Weight))
			.saturating_add(RocksDbWeight::get().writes(8 as Weight))
	}
}